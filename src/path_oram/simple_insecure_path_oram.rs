// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and (non-recursive) position map.

use crate::{
    Address, BucketSizeType, CountAccessesDatabase, Database, Oram, OramBlock, TreeHeight,
    TreeIndex, DEFAULT_BLOCKS_PER_BUCKET, MAXIMUM_TREE_HEIGHT,
};
use rand::{seq::SliceRandom, CryptoRng, Rng, RngCore};
use std::ops::BitAnd;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

use super::{Bucket, CompleteBinaryTreeIndex, PathOramBlock};

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
pub struct SimpleInsecurePathOram<V: OramBlock, const Z: BucketSizeType> {
    /// Again, making the ORAM untrusted storage `physical_memory` public for now, for benchmarking purposes.
    pub physical_memory: CountAccessesDatabase<Bucket<V, Z>>,
    stash: Vec<PathOramBlock<V>>,
    position_map: CountAccessesDatabase<TreeIndex>,
    height: TreeHeight,
}

impl<V: OramBlock, const Z: BucketSizeType> Oram<V> for SimpleInsecurePathOram<V, Z> {
    fn access<R: Rng + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        callback: F,
        rng: &mut R,
    ) -> V {
        let position = self.position_map.read(address);
        let new_position = CompleteBinaryTreeIndex::random_leaf(self.height, rng);
        self.position_map.write(address, new_position);

        self.read_path(position);

        // LEAK: The next three lines leak information
        // about the state of the stash at the end of the access.
        let result = self.access_stash(address, callback, new_position);

        self.write_path(position);

        self.cleanup_stash();

        result
    }

    fn new<R: Rng + CryptoRng>(block_capacity: usize, rng: &mut R) -> Self {
        assert!(block_capacity.is_power_of_two());
        assert!(block_capacity > 1);

        let number_of_nodes = block_capacity;

        let height = block_capacity.ilog2() - 1;
        assert!(height <= MAXIMUM_TREE_HEIGHT);

        let stash: Vec<PathOramBlock<V>> = Vec::new();

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory = CountAccessesDatabase::new(number_of_nodes);

        // Initialize the logical memory to contain 0 at every address
        let permuted_addresses =
            SimpleInsecurePathOram::<V, Z>::random_permutation_of_0_through_n_exclusive(
                block_capacity,
                rng,
            );

        let mut position_map = CountAccessesDatabase::new(block_capacity);

        let first_leaf_index = 2u64.pow(height) as usize;
        let last_leaf_index = (2 * first_leaf_index) - 1;

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
                position_map.write(permuted_addresses[address_index], leaf_index as TreeIndex);
            }
            physical_memory.write(leaf_index, bucket_to_write);
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

impl<V: OramBlock, const Z: BucketSizeType> SimpleInsecurePathOram<V, Z> {
    fn read_path(&mut self, position: TreeIndex) {
        assert!(position.is_leaf(self.height));
        for depth in 0..=self.height {
            let bucket_index: TreeIndex = position.node_on_path(depth, self.height);
            self.read_bucket(bucket_index);
        }
    }

    fn read_bucket(&mut self, bucket_index: TreeIndex) {
        let bucket = self.physical_memory.read(bucket_index as Address);
        for block in bucket.blocks {
            self.stash.push(block);
        }
    }

    fn access_stash<F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        callback: F,
        new_position: TreeIndex,
    ) -> V {
        // let value_to_write = optional_new_value.unwrap_or_else(V::default);
        // let oram_operation_is_write = optional_new_value.is_some();
        let mut result: V = V::default();

        // LEAK: The time taken by this loop leaks the size of the stash
        for block in &mut self.stash {
            let is_requested_index = block.address.ct_eq(&address);

            // Read current value of target block into result
            result.conditional_assign(&block.value, is_requested_index);

            // Write new position into target block
            block
                .position
                .conditional_assign(&new_position, is_requested_index);

            // let should_write = is_requested_index.bitand(oram_operation_is_write);
            // Write new value and position into target block in case of write

            let value_to_write = callback(&result);

            block
                .value
                .conditional_assign(&value_to_write, is_requested_index);
            // block.value = value_to_write;
        }
        result
    }

    // fn access_stash<F: FnOnce(&V) -> V>(
    //     &mut self,
    //     address: Address,
    //     callback: F,
    //     new_position: TreeIndex,
    // ) -> V {
    //     let linear_time_oram = LinearTimeOram
    // }

    fn write_path(&mut self, position: TreeIndex) {
        for depth in (0..=self.height).rev() {
            self.write_bucket(position, depth);
        }
    }

    fn write_bucket(&mut self, position: TreeIndex, depth: TreeHeight) {
        let bucket_address = position.node_on_path(depth, self.height);

        let mut new_bucket: Bucket<V, Z> = Bucket::default();

        for slot_number in 0..DEFAULT_BLOCKS_PER_BUCKET {
            let slot = &mut new_bucket.blocks[slot_number];
            self.write_slot(position, depth, slot);
        }

        self.physical_memory
            .write(bucket_address as usize, new_bucket);
    }

    fn write_slot(&mut self, position: TreeIndex, depth: TreeHeight, slot: &mut PathOramBlock<V>) {
        let mut slot_already_written: Choice = 0.into();

        // LEAK: The time taken by this loop leaks the size of the stash.
        for stash_index in (0..self.stash.len()).rev() {
            let stashed_block: &mut PathOramBlock<V> = &mut self.stash[stash_index];

            let is_dummy = stashed_block.ct_is_dummy();

            // Compute whether the stashed block (if not a dummy block) can be placed in the given slot
            let mut stashed_block_position = stashed_block.position;
            let arbitrary_leaf = 2u64.pow(self.height);
            stashed_block_position.conditional_assign(&arbitrary_leaf, is_dummy);
            let stashed_block_assigned_bucket =
                stashed_block_position.node_on_path(depth, self.height);
            let slot_bucket = position.node_on_path(depth, self.height);
            let position_matches = stashed_block_assigned_bucket.ct_eq(&slot_bucket);

            let should_write = slot_already_written
                .bitand(!is_dummy)
                .bitand(!position_matches);

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
        // Initialize the logical memory to contain 0 at every address
        let permuted_addresses = 0..n;
        let mut permuted_addresses = Vec::from_iter(permuted_addresses);
        let permuted_addresses = permuted_addresses.as_mut_slice();
        permuted_addresses.shuffle(rng);
        Vec::from(permuted_addresses)
    }
}

/// A type alias for a simple `SimpleInsecurePathOram` monomorphization.
pub type ConcreteSimpleInsecurePathOram<V> = SimpleInsecurePathOram<V, DEFAULT_BLOCKS_PER_BUCKET>;

#[cfg(test)]
mod tests {
    use crate::BlockValue;

    use crate::test_utils::{
        create_correctness_test_block_value, create_correctness_tests_for_oram_type,
        create_correctness_tests_for_workload_and_oram_type, test_correctness_linear_workload,
        test_correctness_random_workload,
    };

    use super::ConcreteSimpleInsecurePathOram;

    create_correctness_tests_for_oram_type!(ConcreteSimpleInsecurePathOram);
}
