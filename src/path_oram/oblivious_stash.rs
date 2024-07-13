// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A fixed-size, obliviously accessed stash implemented using oblivious sorting.

use super::{
    bucket::Bucket,
    path_oram_block::PathOramBlock,
    stash::{Stash, StashSize},
    tree_index::CompleteBinaryTreeIndex,
    TreeIndex,
};
use crate::{
    database::Database, path_oram::bitonic_sort::bitonic_sort_by_keys, BucketSize, OramBlock,
};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

#[derive(Debug)]
/// A fixed-size, obliviously accessed stash data structure implemented using oblivious sorting.
pub struct BitonicStash<V: OramBlock> {
    blocks: Vec<PathOramBlock<V>>,
    path_size: StashSize,
}

impl<V: OramBlock> BitonicStash<V> {
    fn len(&self) -> usize {
        self.blocks.len()
    }
}

impl<V: OramBlock> Stash<V> for BitonicStash<V> {
    fn new(path_size: StashSize, overflow_size: StashSize) -> Self {
        Self {
            blocks: vec![PathOramBlock::<V>::dummy(); path_size + overflow_size],
            path_size,
        }
    }

    fn write_to_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: super::TreeIndex,
    ) {
        let height = position.depth();

        let mut level_assignments = vec![TreeIndex::MAX; self.len()];
        let mut level_counts = vec![0; height as usize + 1];

        for (i, block) in self.blocks.iter().enumerate() {
            // If `block` is a dummy, the rest of this loop iteration will be a no-op, and the values don't matter.
            let block_is_dummy = block.ct_is_dummy();

            // Set up valid but meaningless input to the computation in case `block` is a dummy.
            let an_arbitrary_leaf: TreeIndex = 1 << height;
            let block_position =
                TreeIndex::conditional_select(&block.position, &an_arbitrary_leaf, block_is_dummy);

            let block_level = block_position
                .ct_common_ancestor_of_two_leaves(position)
                .depth() as u64;

            // Obliviously scan through the buckets, assigning the block to the correct one, or to the overflow.
            for (level, count) in level_counts.iter_mut().enumerate() {
                let block_level_bucket_full: Choice = count.ct_eq(&(Z as u64));
                let correct_level = level.ct_eq(&(block_level as usize));

                // If the bucket `block` should go in is full, assign the block to the overflow.
                let should_overflow = correct_level & block_level_bucket_full & (!block_is_dummy);
                level_assignments[i].conditional_assign(&(TreeIndex::MAX - 1), should_overflow);

                // Otherwise, assign
                let should_assign = correct_level & (!block_level_bucket_full) & (!block_is_dummy);
                let level_count_incremented = *count + 1;
                count.conditional_assign(&level_count_incremented, should_assign);
                level_assignments[i].conditional_assign(&block_level, should_assign);
            }
        }

        // Assign dummy blocks to the remaining non-full buckets until all buckets are full.
        for (i, block) in self.blocks.iter().enumerate() {
            let block_free = block.ct_is_dummy();

            let mut assigned: Choice = 0.into();
            for (level, count) in level_counts.iter_mut().enumerate() {
                let full = count.ct_eq(&(Z as u64));
                let no_op = assigned | full | !block_free;

                level_assignments[i].conditional_assign(&(level as u64), !no_op);
                count.conditional_assign(&(*count + 1), !no_op);
                assigned |= !no_op;
            }
        }

        bitonic_sort_by_keys(&mut self.blocks, &mut level_assignments);

        // Write the first Z * height blocks into slots in the tree
        for depth in 0..=height {
            let mut new_bucket: Bucket<V, Z> = Bucket::default();

            for slot_number in 0..Z {
                let stash_index = (depth as usize) * Z + slot_number;

                new_bucket.blocks[slot_number] = self.blocks[stash_index];
            }

            physical_memory.write_db(position.node_on_path(depth, height) as usize, new_bucket);
        }
    }

    fn access<F: Fn(&V) -> V>(
        &mut self,
        address: crate::Address,
        new_position: super::TreeIndex,
        value_callback: F,
    ) -> V {
        let mut result: V = V::default();

        for block in &mut self.blocks {
            let is_requested_index = block.address.ct_eq(&address);

            // Read current value of target block into `result`.
            result.conditional_assign(&block.value, is_requested_index);

            // Write new position into target block.
            block
                .position
                .conditional_assign(&new_position, is_requested_index);

            // If a write, write new value into target block.
            let value_to_write = value_callback(&result);

            block
                .value
                .conditional_assign(&value_to_write, is_requested_index);
        }
        result
    }

    #[cfg(test)]
    fn occupancy(&self) -> StashSize {
        // self.high_water_mark as usize
        let mut result = 0;
        // for block in &self.blocks {
        for i in self.path_size..self.blocks.len() {
            if !self.blocks[i].is_dummy() {
                result += 1;
            }
        }
        result
    }

    fn read_from_path<const Z: crate::BucketSize, T: crate::database::Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        position: super::TreeIndex,
    ) {
        let height = position.depth();

        for i in (0..(self.path_size / Z) as u32).rev() {
            let bucket_index = position.node_on_path(i, height);
            let bucket = physical_memory.read_db(bucket_index as usize);
            for slot_index in 0..Z {
                self.blocks[Z * (i as usize) + slot_index] = bucket.blocks[slot_index];
            }
        }
    }
}
