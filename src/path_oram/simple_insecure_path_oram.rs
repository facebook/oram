// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and (non-recursive) position map.

use super::{
    bucket::Bucket, tree_index::CompleteBinaryTreeIndex, PathOramBlock, TreeHeight, TreeIndex,
    MAXIMUM_TREE_HEIGHT,
};
use crate::{
    database::{CountAccessesDatabase, Database},
    Address, BucketSize, Oram, OramBlock,
};
use rand::{seq::SliceRandom, CryptoRng, Rng, RngCore};
use std::{mem, ops::BitAnd};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

/// Specialization
pub type SimpleInsecurePathOram<V, const Z: BucketSize> =
    VecPathOram<V, Z, CountAccessesDatabase<TreeIndex>>;

/// (!) This is a development stepping stone, not a finished implementation. (!)
/// A simple, insecure implementation of Path ORAM
/// whose stash is just a `Vec` of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// the only leakage would be the size of the stash
/// and the positions of dummy blocks in the stash at the end of each access.
/// (Such leakage would likely still be unacceptable.)
/// The leakage will be addressed by more sophisticated stash access routines
/// in one of the next few iterations.
#[derive(Debug)]
pub struct VecPathOram<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex>> {
    // The fields below are not meant to be exposed to clients. They are public for benchmarking and testing purposes.
    /// The underlying untrusted memory that the ORAM is obliviously accessing on behalf of its client.
    pub physical_memory: CountAccessesDatabase<Bucket<V, Z>>,
    /// The Path ORAM stash.
    pub stash: Vec<PathOramBlock<V>>,
    /// The Path ORAM position map.
    pub position_map: P,
    /// The height of the Path ORAM tree data structure.
    pub height: TreeHeight,
}

impl<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex> + std::fmt::Debug> Oram<V>
    for VecPathOram<V, Z, P>
{
    fn access<R: Rng + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        callback: F,
        rng: &mut R,
    ) -> V {
        assert!(address < self.block_capacity());

        // Get the position of the target block (with address `address`),
        // and update that block's position map entry to a fresh random position
        let new_position = CompleteBinaryTreeIndex::random_leaf(self.height, rng);
        debug_assert_ne!(new_position, 0);
        let position = self.position_map.write(address, new_position, rng);
        debug_assert_ne!(position, 0);

        self.read_path_into_stash(position);

        // LEAK: The next three lines leak information
        // about the state of the stash at the end of the access.

        // Scan the stash for the target block, read its value into `result`,
        // and overwrite its position (and possibly its value).
        let result = self.access_stash(address, callback, new_position, rng);

        // Evict blocks from the stash into the path that was just read,
        // replacing them with dummy blocks.
        self.write_stash_into_tree_path(position);

        // Remove accumulated dummy blocks from the stash.
        self.cleanup_stash();

        result
    }

    fn new<R: Rng + CryptoRng>(block_capacity: usize, rng: &mut R) -> Self {
        log::debug!(
            "Oram::new -- BlockOram(B = {}, Z = {}, C = {})",
            mem::size_of::<V>(),
            Z,
            block_capacity
        );

        assert!(block_capacity.is_power_of_two());
        assert!(block_capacity > 1);

        let number_of_nodes = block_capacity;

        let height = block_capacity.ilog2() - 1;
        assert!(height <= MAXIMUM_TREE_HEIGHT);

        let stash: Vec<PathOramBlock<V>> = Vec::new();

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory: CountAccessesDatabase<Bucket<V, Z>> =
            Database::new(number_of_nodes);

        // The rest of this function initializes the logical memory to contain default values at every address.
        // This is done by (1) initializing the position map with fresh random leaf identifiers,
        // and (2) writing blocks to the physical memory with the appropriate positions, and default values.

        let mut position_map = P::new(block_capacity, rng);

        let permuted_addresses =
            SimpleInsecurePathOram::<V, Z>::random_permutation_of_0_through_n_exclusive(
                block_capacity,
                rng,
            );

        let first_leaf_index = 2u64.pow(height) as usize;
        let last_leaf_index = (2 * first_leaf_index) - 1;

        // Iterate over leaves, writing 2 blocks into each leaf bucket with random(ly permuted) addresses and default values.
        let addresses_per_leaf = 2;
        for leaf_index in first_leaf_index..=last_leaf_index {
            let mut bucket_to_write = Bucket::<V, Z>::default();
            for slot_index in 0..addresses_per_leaf {
                let address_index = (leaf_index - first_leaf_index) * 2 + slot_index;
                bucket_to_write.blocks[slot_index] = PathOramBlock::<V> {
                    value: V::default(),
                    address: permuted_addresses[address_index],
                    position: leaf_index as TreeIndex,
                };

                position_map.write(
                    permuted_addresses[address_index],
                    leaf_index as TreeIndex,
                    rng,
                );
            }

            // Write the leaf bucket back to physical memory.
            physical_memory.write_db(leaf_index, bucket_to_write);
        }

        Self {
            physical_memory,
            stash,
            position_map,
            height,
        }
    }

    fn block_capacity(&self) -> Address {
        self.physical_memory.capacity()
    }
}

impl<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex>> VecPathOram<V, Z, P> {
    fn read_path_into_stash(&mut self, position: TreeIndex) {
        assert!(position.is_leaf(self.height));
        for depth in 0..=self.height {
            let bucket_index: TreeIndex = position.node_on_path(depth, self.height);
            self.read_bucket_into_stash(bucket_index);
        }
    }

    fn read_bucket_into_stash(&mut self, bucket_index: TreeIndex) {
        let bucket = self.physical_memory.read_db(bucket_index as Address);
        for block in bucket.blocks {
            self.stash.push(block);
        }
    }

    fn access_stash<R: CryptoRng + RngCore, F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        callback: F,
        new_position: TreeIndex,
        _: &mut R,
    ) -> V {
        let mut result: V = V::default();

        // LEAK: The time taken by this loop leaks the size of the stash
        for block in &mut self.stash {
            let is_requested_index = block.address.ct_eq(&address);

            // Read current value of target block into `result``.
            result.conditional_assign(&block.value, is_requested_index);

            // Write new position into target block.
            block
                .position
                .conditional_assign(&new_position, is_requested_index);

            // If a write, write new value into target block.
            let value_to_write = callback(&result);

            block
                .value
                .conditional_assign(&value_to_write, is_requested_index);
        }
        result
    }

    // Iterate over each slot of each bucket on the path.
    // For each slot, make a linear scan over the stash, obliviously writing the first block
    // (if any) with the appropriate position into the slot, and
    // overwriting that block in the stash with a dummy block.
    fn write_stash_into_tree_path(&mut self, position: TreeIndex) {
        for depth in (0..=self.height).rev() {
            self.write_stash_into_bucket(position, depth);
        }
    }

    fn write_stash_into_bucket(&mut self, position: TreeIndex, depth: TreeHeight) {
        let bucket_address = position.node_on_path(depth, self.height);

        let mut new_bucket: Bucket<V, Z> = Bucket::default();

        for slot_number in 0..Z {
            let slot = &mut new_bucket.blocks[slot_number];
            self.write_stash_into_slot(position, depth, slot);
        }

        self.physical_memory
            .write_db(bucket_address as usize, new_bucket);
    }

    fn write_stash_into_slot(
        &mut self,
        position: TreeIndex,
        depth: TreeHeight,
        slot: &mut PathOramBlock<V>,
    ) {
        let mut slot_already_written: Choice = 0.into();

        // LEAK: The time taken by this loop leaks the size of the stash.
        for stash_index in (0..self.stash.len()).rev() {
            let stashed_block: &mut PathOramBlock<V> = &mut self.stash[stash_index];

            let is_dummy = stashed_block.ct_is_dummy();

            // Compute whether the stashed block can be placed in the given slot.
            // The result of this computation is arbitrary if the stashed block is a dummy block.
            let mut stashed_block_position = stashed_block.position;
            let arbitrary_leaf = 2u64.pow(self.height);
            stashed_block_position.conditional_assign(&arbitrary_leaf, is_dummy);
            let stashed_block_assigned_bucket =
                stashed_block_position.node_on_path(depth, self.height);
            let slot_bucket = position.node_on_path(depth, self.height);
            let position_matches = stashed_block_assigned_bucket.ct_eq(&slot_bucket);

            // This slot should be written with `stashed_block` if
            // (1) this slot has not previously been written
            // (2) the stashed block is not a dummy block
            // (3) the stashed block's position is such that it can be placed in the slot.
            let should_write = (!slot_already_written)
                .bitand(!is_dummy)
                .bitand(position_matches);

            // Conditionally write the slot, replace the stashed block with a dummy block,
            // and flag the write.
            slot.conditional_assign(stashed_block, should_write);
            stashed_block.conditional_assign(&PathOramBlock::dummy(), should_write);
            slot_already_written.conditional_assign(&(1.into()), should_write);
        }
    }

    // LEAK: The behavior of this loop leaks the size of the stash and the location of dummy blocks in the stash.
    fn cleanup_stash(&mut self) {
        for stash_index in (0..self.stash.len()).rev() {
            if self.stash[stash_index].ct_is_dummy().into() {
                self.stash.remove(stash_index);
                continue;
            }
        }
    }

    fn random_permutation_of_0_through_n_exclusive<R: RngCore + CryptoRng>(
        n: Address,
        rng: &mut R,
    ) -> Vec<Address> {
        let permuted_addresses = 0..n;
        let mut permuted_addresses = Vec::from_iter(permuted_addresses);
        let permuted_addresses = permuted_addresses.as_mut_slice();
        permuted_addresses.shuffle(rng);
        Vec::from(permuted_addresses)
    }
}

/// A type alias for a simple `SimpleInsecurePathOram` monomorphization.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_value::*, path_oram::*, test_utils::*, *};
    use std::iter::zip;

    pub type ConcreteSimpleInsecurePathOram<const B: BlockSize, V> =
        SimpleInsecurePathOram<V, DEFAULT_BLOCKS_PER_BUCKET>;

    create_correctness_tests_for_oram_type!(ConcreteSimpleInsecurePathOram, BlockValue);

    // Test that the stash size is not growing too large.
    type SipoStashSizeMonitor<const B: BlockSize, V> =
        StashSizeMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoStashSizeMonitor, BlockValue);

    // Test that the total number of non-dummy blocks in the ORAM stays constant.
    type SipoConstantOccupancyMonitor<const B: BlockSize, V> =
        ConstantOccupancyMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoConstantOccupancyMonitor, BlockValue);

    // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
    type SipoCountPhysicalAccessesMonitor<const B: BlockSize, V> =
        PhysicalAccessCountMonitor<ConcreteSimpleInsecurePathOram<B, V>>;
    create_correctness_tests_for_oram_type!(SipoCountPhysicalAccessesMonitor, BlockValue);

    // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
    #[derive(Debug)]
    struct SipoAccessDistributionTester<const B: BlockSize, V: OramBlock> {
        oram: ConcreteSimpleInsecurePathOram<B, V>,
    }
    create_statistics_test_for_oram_type!(SipoAccessDistributionTester, BlockValue);
}
