// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Path ORAM.

use super::{position_map::PositionMap, stash::ObliviousStash};
use crate::{
    bucket::{Bucket, PathOramBlock, PositionBlock},
    database::{CountAccessesDatabase, Database},
    utils::{
        invert_permutation_oblivious, random_permutation_of_0_through_n_exclusive, to_usize_vec,
        CompleteBinaryTreeIndex, TreeHeight,
    },
    Address, BlockSize, BucketSize, Oram, OramBlock, OramError, RecursionCutoff, StashSize,
};
use rand::{CryptoRng, Rng};
use std::mem;

/// The default cutoff size in blocks
/// below which `PathOram` uses a linear position map instead of a recursive one.
pub const DEFAULT_RECURSION_CUTOFF: RecursionCutoff = 1 << 12;

/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSize = 4;

/// The default number of positions stored per position block.
pub const DEFAULT_POSITIONS_PER_BLOCK: BlockSize = 512;

/// The default number of overflow blocks that the Path ORAM stash (and recursive stashes) can store.
pub const DEFAULT_STASH_OVERFLOW_SIZE: StashSize = 40;

/// A doubly oblivious Path ORAM.
///
/// ## Parameters
///
/// - `V`: the type (block) stored by the ORAM.
/// - `Z`: The number of blocks per Path ORAM bucket.
///     Typical values are 3, 4, or 5. Along with SO, this value affects the probability
///     of stash overflow (see below) and should be set with care.
/// - `AB`:
///     The number of positions stored in each block of the recursive position map ORAM.
///     Must be a power of two and must be at least 2 (otherwise the recursion will not terminate).
///     Otherwise, can be freely tuned for performance.
///     Larger `AB` means fewer levels of recursion, but each level is more expensive.
///     The default setting of 512 results in approximately 4K blocks (but see Issue #46).
/// - `RT`: The recursion threshold. If the number of position blocks is at most this value,
///     the position map will be a linear scanning ORAM; otherwise it will be a recursive Path ORAM.
///     Can be freely tuned for performance.
///     Larger `RT` means fewer levels of recursion, but the base position map is more expensive.
/// - `SO`: The number of blocks that the stash can store between ORAM accesses without overflowing.
///     This value affects the probability of stash overflow (see below) and should be set with care.
///
/// ## Security
///
/// ORAM operations are guaranteed to be oblivious, *unless* the stash overflows.
/// In this case, the stash will grow, which reveals that the overflow occurred.
/// This is a violation of obliviousness, but a mild one in several ways.
/// The stash overflow is very likely to reset to empty after the overflow,
/// and stash overflows are isolated events. It is not at all obvious
/// how an attacker might use a stash overflow to infer properties of the access pattern.
///
/// That said, it is best to choose parameters so that the stash does not ever overflow.
/// With Z = 4, experiments from the [original Path ORAM paper](https://eprint.iacr.org/2013/280.pdf)
/// indicate that the probability of overflow is independent of the number N of blocks stored,
/// and that setting SO = 40 is enough to reduce this probability to below 2^{-50} (Figure 3).
/// The authors conservatively estimate that setting SO = 89 suffices for 2^{-80} overflow probability.
/// The choice Z = 3 is also popular, although the probability of overflow is less well understood.
#[derive(Debug)]
pub struct PathOram<
    V: OramBlock,
    const Z: BucketSize,
    const AB: BlockSize,
    const RT: RecursionCutoff,
    const SO: StashSize,
> {
    // The fields below are not meant to be exposed to clients. They are public for benchmarking and testing purposes.
    /// The underlying untrusted memory that the ORAM is obliviously accessing on behalf of its client.
    pub physical_memory: CountAccessesDatabase<Bucket<V, Z>>,
    /// The Path ORAM stash.
    pub stash: ObliviousStash<V>,
    /// The Path ORAM position map.
    pub position_map: PositionMap<AB, Z, RT, SO>,
    /// The height of the Path ORAM tree data structure.
    pub height: TreeHeight,
}

/// An `Oram` suitable for most use cases, with reasonable default choices of parameters.
pub type DefaultOram<V> = PathOram<
    V,
    DEFAULT_BLOCKS_PER_BUCKET,
    DEFAULT_POSITIONS_PER_BLOCK,
    DEFAULT_RECURSION_CUTOFF,
    DEFAULT_STASH_OVERFLOW_SIZE,
>;

impl<
        V: OramBlock,
        const Z: BucketSize,
        const AB: BlockSize,
        const RT: RecursionCutoff,
        const SO: StashSize,
    > Oram<V> for PathOram<V, Z, AB, RT, SO>
{
    fn access<R: Rng + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        callback: F,
        rng: &mut R,
    ) -> Result<V, OramError> {
        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not.
        if address > self.block_capacity()? {
            return Err(OramError::AddressOutOfBoundsError);
        }

        // Get the position of the target block (with address `address`),
        // and update that block's position map entry to a fresh random position
        let new_position = CompleteBinaryTreeIndex::random_leaf(self.height, rng)?;
        let position = self.position_map.write(address, new_position, rng)?;

        assert!(position.is_leaf(self.height));

        self.stash
            .read_from_path(&mut self.physical_memory, position)?;

        // Scan the stash for the target block, read its value into `result`,
        // and overwrite its position (and possibly its value).
        let result = self.stash.access(address, new_position, callback);

        // Evict blocks from the stash into the path that was just read,
        // replacing them with dummy blocks.
        self.stash
            .write_to_path(&mut self.physical_memory, position)?;

        result
    }

    fn new<R: Rng + CryptoRng>(block_capacity: Address, rng: &mut R) -> Result<Self, OramError> {
        log::debug!(
            "Oram::new -- BlockOram(B = {}, Z = {}, C = {})",
            mem::size_of::<V>(),
            Z,
            block_capacity
        );

        if !block_capacity.is_power_of_two() | (block_capacity <= 1) {
            return Err(OramError::InvalidConfigurationError);
        }

        let number_of_nodes = block_capacity;

        let height: u64 = (block_capacity.ilog2() - 1).into();

        let path_size = u64::try_from(Z)? * (height + 1);
        let stash = ObliviousStash::new(path_size, SO)?;

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory: CountAccessesDatabase<Bucket<V, Z>> =
            Database::new(number_of_nodes)?;

        // The rest of this function initializes the logical memory to contain default values at every address.
        // This is done by (1) initializing the position map with fresh random leaf identifiers,
        // and (2) writing blocks to the physical memory with the appropriate positions, and default values.
        let mut position_map = PositionMap::new(block_capacity, rng)?;

        let slot_indices_to_addresses =
            random_permutation_of_0_through_n_exclusive(block_capacity, rng);
        let addresses_to_slot_indices = invert_permutation_oblivious(&slot_indices_to_addresses)?;
        let slot_indices_to_addresses = to_usize_vec(slot_indices_to_addresses)?;
        let mut addresses_to_slot_indices = to_usize_vec(addresses_to_slot_indices)?;

        let first_leaf_index: usize = 2u64.pow(height.try_into()?).try_into()?;
        let last_leaf_index = (2 * first_leaf_index) - 1;

        // Iterate over leaves, writing 2 blocks into each leaf bucket with random(ly permuted) addresses and default values.
        let addresses_per_leaf = 2;
        for leaf_index in first_leaf_index..=last_leaf_index {
            let mut bucket_to_write = Bucket::<V, Z>::default();
            for slot_index in 0..addresses_per_leaf {
                let address_index = (leaf_index - first_leaf_index) * 2 + slot_index;
                bucket_to_write.blocks[slot_index] = PathOramBlock::<V> {
                    value: V::default(),
                    address: slot_indices_to_addresses[address_index].try_into()?,
                    position: leaf_index.try_into()?,
                };
            }

            // Write the leaf bucket back to physical memory.
            physical_memory.write_db(leaf_index.try_into()?, bucket_to_write)?;
        }

        // The address block size might not divide the block capacity.
        // If it doesn't, we will have one block that contains dummy values.
        let ab_address: Address = AB.try_into()?;
        let mut num_blocks = block_capacity / ab_address;
        if block_capacity % ab_address > 0 {
            num_blocks += 1;
            addresses_to_slot_indices.resize((block_capacity + ab_address).try_into()?, 0);
        }

        for block_index in 0..num_blocks {
            let mut data = [0; AB];
            for i in 0..AB {
                let offset: usize = (block_index * ab_address).try_into()?;
                data[i] =
                    (first_leaf_index + addresses_to_slot_indices[offset + i] / 2).try_into()?;
            }
            let block = PositionBlock::<AB> { data };
            position_map.write_position_block(block_index * ab_address, block, rng)?;
        }

        Ok(Self {
            physical_memory,
            stash,
            position_map,
            height,
        })
    }

    fn block_capacity(&self) -> Result<Address, OramError> {
        self.physical_memory.capacity()
    }
}

#[cfg(test)]
mod block_oram_tests {
    use bucket::BlockValue;

    use super::*;

    use crate::test_utils::*;
    use crate::*;
    use core::iter::zip;

    pub type ConcreteObliviousBlockOram<const B: BlockSize, V> = PathOram<
        V,
        DEFAULT_BLOCKS_PER_BUCKET,
        B,
        DEFAULT_RECURSION_CUTOFF,
        DEFAULT_STASH_OVERFLOW_SIZE,
    >;

    create_correctness_tests_for_oram_type!(ConcreteObliviousBlockOram, BlockValue);

    // Test that the stash size is not growing too large.
    type COBOStashSizeMonitor<const AB: BlockSize, V> =
        StashSizeMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type COBOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type COBOCountPhysicalAccessesMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteObliviousBlockOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COBOCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct COBOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteObliviousBlockOram<B, V>,
    }
    create_statistics_test_for_oram_type!(COBOAccessDistributionTester, BlockValue);
}

#[cfg(test)]
mod address_oram_tests {
    use bucket::BlockValue;

    use super::*;
    use crate::*;
    use crate::{test_utils::*, OramBlock};
    use core::iter::zip;

    type ConcreteObliviousAddressOram<const AB: BlockSize, V> = PathOram<
        V,
        DEFAULT_BLOCKS_PER_BUCKET,
        AB,
        DEFAULT_RECURSION_CUTOFF,
        DEFAULT_STASH_OVERFLOW_SIZE,
    >;

    create_correctness_tests_for_oram_type!(ConcreteObliviousAddressOram, PositionBlock);

    // Test that the stash size is not growing too large.
    type COAOStashSizeMonitor<const AB: BlockSize, V> =
        StashSizeMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOStashSizeMonitor, PositionBlock);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type COAOConstantOccupancyMonitor<const AB: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOConstantOccupancyMonitor, PositionBlock);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type COAOPhysicalAccessCountMonitor<const AB: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteObliviousAddressOram<AB, V>>;
    create_correctness_tests_for_oram_type!(COAOPhysicalAccessCountMonitor, PositionBlock);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct COAOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteObliviousAddressOram<B, V>,
    }
    create_statistics_test_for_oram_type!(COAOAccessDistributionTester, BlockValue);
}
