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
    linear_time_oram::LinearTimeOram,
    utils::{
        invert_permutation_oblivious, random_permutation_of_0_through_n_exclusive, to_usize_vec,
        CompleteBinaryTreeIndex, TreeHeight,
    },
    Address, BlockSize, BucketSize, Oram, OramBlock, OramError, RecursionCutoff, StashSize,
};
use rand::{CryptoRng, Rng};

/// The default cutoff size in blocks
/// below which `PathOram` uses a linear position map instead of a recursive one.
pub const DEFAULT_RECURSION_CUTOFF: RecursionCutoff = 1 << 14;

/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSize = 4;

/// The default number of positions stored per position block.
pub const DEFAULT_POSITIONS_PER_BLOCK: BlockSize = 8;

/// The default number of overflow blocks that the Path ORAM stash (and recursive stashes) can store.
pub const DEFAULT_STASH_OVERFLOW_SIZE: StashSize = 40;

const LINEAR_TIME_ORAM_CUTOFF: RecursionCutoff = 1 << 10;

/// A doubly oblivious Path ORAM.
///
/// ## Parameters
///
/// - Block type `V`: the type of elements stored by the ORAM.
/// - Bucket size `Z`: the number of blocks per Path ORAM bucket.
///     Must be at least 2. Typical values are 3, 4, or 5.
///     Along with the overflow size, this value affects the probability
///     of stash overflow (see below) and should be set with care.
/// - Positions per block `AB`:
///     The number of positions stored in each block of the recursive position map ORAM.
///     Must be a power of two and must be at least 2 (otherwise the recursion will not terminate).
///     Otherwise, can be freely tuned for performance.
///     Larger `AB` means fewer levels of recursion but higher costs for accessing each level.
/// - Recursion threshold: the maximum number of position blocks that will be stored in a recursive Path ORAM.
///     Below this value, the position map will be a linear scanning ORAM.
///     Can be freely tuned for performance.
///     A larger values means fewer levels of recursion, but a more expensive base position map.
/// - Overflow size: The number of blocks that the stash can store between ORAM accesses without overflowing.
///     Along with the bucket size, this value affects the probability of stash overflow (see below)
///     and should be set with care.
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
pub struct PathOram<V: OramBlock, const Z: BucketSize, const AB: BlockSize> {
    /// The underlying untrusted memory that the ORAM is obliviously accessing on behalf of its client.
    physical_memory: Vec<Bucket<V, Z>>,
    /// The Path ORAM stash.
    stash: ObliviousStash<V>,
    /// The Path ORAM position map.
    position_map: PositionMap<AB, Z>,
    /// The height of the Path ORAM tree data structure.
    height: TreeHeight,
}

/// An `Oram` suitable for most use cases, with reasonable default choices of parameters.
#[derive(Debug)]
pub struct DefaultOram<V: OramBlock>(DefaultOramBackend<V>);

#[derive(Debug)]
enum DefaultOramBackend<V: OramBlock> {
    Path(PathOram<V, DEFAULT_BLOCKS_PER_BUCKET, DEFAULT_POSITIONS_PER_BLOCK>),
    Linear(LinearTimeOram<V>),
}

impl<V: OramBlock> Oram for DefaultOram<V> {
    type V = V;

    fn block_capacity(&self) -> Result<Address, OramError> {
        match &self.0 {
            DefaultOramBackend::Path(p) => p.block_capacity(),
            DefaultOramBackend::Linear(l) => l.block_capacity(),
        }
    }

    fn access<R: rand::RngCore + CryptoRng, F: Fn(&Self::V) -> Self::V>(
        &mut self,
        index: Address,
        callback: F,
        rng: &mut R,
    ) -> Result<Self::V, OramError> {
        match &mut self.0 {
            DefaultOramBackend::Path(p) => p.access(index, callback, rng),
            DefaultOramBackend::Linear(l) => l.access(index, callback, rng),
        }
    }
}

impl<V: OramBlock> DefaultOram<V> {
    /// Returns a new ORAM mapping addresses `0 <= address < block_capacity` to default `V` values.
    ///
    /// # Errors
    ///
    /// If `block_capacity` is not a power of two, returns an `InvalidConfigurationError`.
    pub fn new<R: Rng + CryptoRng>(
        block_capacity: Address,
        rng: &mut R,
    ) -> Result<Self, OramError> {
        if block_capacity < LINEAR_TIME_ORAM_CUTOFF {
            Ok(Self(DefaultOramBackend::Linear(LinearTimeOram::new(
                block_capacity,
            )?)))
        } else {
            Ok(Self(DefaultOramBackend::Path(PathOram::<
                V,
                DEFAULT_BLOCKS_PER_BUCKET,
                DEFAULT_POSITIONS_PER_BLOCK,
            >::new_with_parameters(
                block_capacity,
                rng,
                DEFAULT_STASH_OVERFLOW_SIZE,
                DEFAULT_RECURSION_CUTOFF,
            )?)))
        }
    }
}

impl<V: OramBlock, const Z: BucketSize, const AB: BlockSize> PathOram<V, Z, AB> {
    /// Returns a new `PathOram` mapping addresses `0 <= address < block_capacity` to default `V` values,
    /// with a stash overflow size of `overflow_size` blocks, and a recursion cutoff of `recursion_cutoff`.
    /// (See [`PathOram`]) for a description of these parameters).
    ///
    /// # Errors
    ///
    /// Returns an `InvalidConfigurationError` in the following cases.
    ///
    /// - `block_capacity` is 0, 1, or is not a power of two.
    /// - `AB` is 0, 1, or is not a power of two.
    /// - `Z` is 0 or 1.
    /// - `recursion_cutoff` is 0.
    ///
    /// If `block_capacity` is not a power of two, returns an `InvalidConfigurationError`.
    pub fn new_with_parameters<R: Rng + CryptoRng>(
        block_capacity: Address,
        rng: &mut R,
        overflow_size: StashSize,
        recursion_cutoff: RecursionCutoff,
    ) -> Result<Self, OramError> {
        log::info!("PathOram::new(capacity = {})", block_capacity,);

        if !block_capacity.is_power_of_two() | (block_capacity <= 1) {
            return Err(OramError::InvalidConfigurationError);
        }

        if Z <= 1 {
            return Err(OramError::InvalidConfigurationError);
        }

        if recursion_cutoff == 0 {
            return Err(OramError::InvalidConfigurationError);
        }

        let number_of_nodes = block_capacity;

        let height: u64 = (block_capacity.ilog2() - 1).into();

        let path_size = u64::try_from(Z)? * (height + 1);
        let stash = ObliviousStash::new(path_size, overflow_size)?;

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory = Vec::new();
        physical_memory.resize(usize::try_from(number_of_nodes)?, Bucket::<V, Z>::default());

        // The rest of this function initializes the logical memory to contain default values at every address.
        // This is done by (1) initializing the position map with fresh random leaf identifiers,
        // and (2) writing blocks to the physical memory with the appropriate positions, and default values.
        let mut position_map =
            PositionMap::new(block_capacity, rng, overflow_size, recursion_cutoff)?;

        let slot_indices_to_addresses =
            random_permutation_of_0_through_n_exclusive(block_capacity, rng);
        let addresses_to_slot_indices = invert_permutation_oblivious(&slot_indices_to_addresses)?;
        let slot_indices_to_addresses = to_usize_vec(slot_indices_to_addresses)?;
        let mut addresses_to_slot_indices = to_usize_vec(addresses_to_slot_indices)?;

        let first_leaf_index: usize = 2u64.pow(height.try_into()?).try_into()?;
        let last_leaf_index = (2 * first_leaf_index) - 1;

        // Iterate over leaves, writing 2 blocks into each leaf bucket with random(ly permuted) addresses and default values.
        let addresses_per_leaf = 2;
        for (leaf_index, tree_bucket) in physical_memory
            .iter_mut()
            .enumerate()
            .take(last_leaf_index + 1)
            .skip(first_leaf_index)
        {
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
            *tree_bucket = bucket_to_write;
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

    #[cfg(test)]
    pub(crate) fn stash_occupancy(&self) -> StashSize {
        self.stash.occupancy()
    }
}

impl<V: OramBlock, const Z: BucketSize, const AB: BlockSize> Oram for PathOram<V, Z, AB> {
    type V = V;

    // REVIEW NOTE: This function has not been modified.
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

    fn block_capacity(&self) -> Result<Address, OramError> {
        Ok(u64::try_from(self.physical_memory.len())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{bucket::*, test_utils::*};

    use rand::{rngs::StdRng, SeedableRng};

    // Test default parameters. For the small capacity used in the tests, this means a linear position map.
    create_path_oram_correctness_tests!(4, 8, 16384, 40);

    // The remaining tests have RECURSION_CUTOFF = 1 in order to test the recursive position map.

    // Default parameters, but with RECURSION_CUTOFF = 1.
    create_path_oram_correctness_tests!(4, 8, 1, 40);

    // Test small initial stash sizes and correct resizing of stash on overflow.
    create_path_oram_correctness_tests!(4, 8, 1, 10);
    create_path_oram_correctness_tests!(4, 8, 1, 0);

    // Test small and large bucket sizes.
    create_path_oram_correctness_tests!(3, 8, 1, 40);
    create_path_oram_correctness_tests!(5, 8, 1, 40);

    // Test small and large position map blocks.
    create_path_oram_correctness_tests!(4, 2, 1, 40);
    create_path_oram_correctness_tests!(4, 64, 1, 40);

    // "Running sanity checks" for the default parameters.

    // Check that the stash size stays reasonably small over the test runs.
    create_path_oram_stash_size_tests!(4, 8, 16384, 40);

    // Sanity checks on the `DefaultOram` convenience wrapper.
    #[test]
    fn default_oram_linear_correctness() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut oram = DefaultOram::<BlockValue<1>>::new(64, &mut rng).unwrap();
        match oram.0 {
            DefaultOramBackend::Linear(_) => {}
            DefaultOramBackend::Path(_) => assert!(false),
        }
        random_workload(&mut oram, 1000);
    }

    // This test is #[ignore]'d because it takes about 1 second to run.
    #[test]
    #[ignore]
    fn default_oram_path_correctness() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut oram = DefaultOram::<BlockValue<1>>::new(2048, &mut rng).unwrap();
        match oram.0 {
            DefaultOramBackend::Linear(_) => {
                assert!(false)
            }
            DefaultOramBackend::Path(_) => {}
        }
        random_workload(&mut oram, 1000);
    }
}
