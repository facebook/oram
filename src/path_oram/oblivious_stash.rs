// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A fixed-size, obliviously accessed stash implemented using oblivious sorting.

// Overriding Clippy's judgment and asserting that in this case,
// a range loop is the best way to convey the intent of the code.
#![allow(clippy::needless_range_loop)]

use super::{
    bucket::Bucket,
    path_oram_block::PathOramBlock,
    stash::{Stash, StashSize},
    tree_index::CompleteBinaryTreeIndex,
};
use crate::{path_oram::bitonic_sort::bitonic_sort_by_keys, OramBlock};
use std::iter::zip;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

#[derive(Debug)]
/// A fixed-size, obliviously accessed stash implemented using oblivious sorting.
pub struct BitonicStash<V: OramBlock> {
    // The blocks in the stash are blocks[0]...blocks[high_water_mark-1].
    blocks: Vec<PathOramBlock<V>>,
    high_water_mark: u64,
}

impl<V: OramBlock> Stash<V> for BitonicStash<V> {
    fn new(capacity: StashSize) -> Self {
        Self {
            blocks: vec![PathOramBlock::<V>::dummy(); capacity],
            high_water_mark: 0,
        }
    }

    fn add_block(&mut self, block: PathOramBlock<V>) {
        assert!(self.high_water_mark < self.blocks.len() as u64);
        self.blocks[self.high_water_mark as usize] = block;
        self.high_water_mark
            .conditional_assign(&(self.high_water_mark + 1), !(block.ct_is_dummy()));
    }

    fn write_to_path<
        const Z: crate::BucketSize,
        T: crate::database::Database<super::bucket::Bucket<V, Z>>,
    >(
        &mut self,
        physical_memory: &mut T,
        height: super::TreeHeight,
        position: super::TreeIndex,
    ) {
        // Sort the blocks to get all the real blocks at the beginning.
        // An optimized version of this function probably does not require this additional sort.
        let mut is_dummy: Vec<u8> = vec![0u8; self.blocks.len()];
        for (a, b) in zip(&self.blocks, &mut is_dummy) {
            b.conditional_assign(&1u8, a.ct_is_dummy());
        }
        bitonic_sort_by_keys(&mut self.blocks, &mut is_dummy);

        let retained_block_depth = (height + 1) as u64;
        let remaining_empty_block_depth = retained_block_depth + 1;

        let mut depth_assignments = vec![remaining_empty_block_depth; self.blocks.len()];
        let mut num_blocks_assigned_to_bucket_at_depth = vec![0u64; (height + 1) as usize];

        let mut num_blocks_retained: u64 = 0;

        // Assign each block to a bucket in the tree, represented by its depth.
        // If the bucket is already full, assign the block a dummy depth value (meaning it will stay in the stash)
        for i in 0..self.high_water_mark as usize {
            assert!(!self.blocks[i].is_dummy());

            assert!(self.blocks[i].position.is_leaf(height));
            assert!(position.is_leaf(height));

            let level = self.blocks[i]
                .position
                .common_ancestor_of_two_leaves(position)
                .depth() as usize;
            let occupancy = num_blocks_assigned_to_bucket_at_depth[level];

            assert!(occupancy <= (Z as u64));
            let bucket_full: Choice = occupancy.ct_eq(&(Z as u64));

            let num_blocks_incremented = num_blocks_assigned_to_bucket_at_depth[level] + 1;
            num_blocks_assigned_to_bucket_at_depth[level]
                .conditional_assign(&(num_blocks_incremented), !bucket_full);
            depth_assignments[i] =
                u64::conditional_select(&(level as u64), &retained_block_depth, bucket_full);

            num_blocks_retained.conditional_assign(&(num_blocks_retained + 1), bucket_full);
        }

        // Fill up the remaining slots in the buckets with dummy blocks.
        let mut i = self.high_water_mark as usize;
        for level in 0..=height as usize {
            for _free_slot in (num_blocks_assigned_to_bucket_at_depth[level] as usize)..Z {
                depth_assignments[i] = level as u64;
                i += 1;
            }
        }

        // Sort the blocks by their level assignments.
        // This puts the blocks assigned to buckets in the first Z * height places, in ascending order of depth,
        // followed by the `num_blocks_retained` blocks retained in the stash, followed by dummy blocks.
        bitonic_sort_by_keys(&mut self.blocks, &mut depth_assignments);

        // Write the first Z * height blocks into slots in the tree
        for depth in 0..=height {
            let mut new_bucket: Bucket<V, Z> = Bucket::default();

            for slot_number in 0..Z {
                let stash_index = (depth as usize) * Z + slot_number;

                new_bucket.blocks[slot_number] = self.blocks[stash_index];

                let shift_from_index = stash_index + Z * ((height + 1) as usize);

                // Acceptable if: does not depend on secret values.
                if shift_from_index >= self.blocks.len() {
                    self.blocks[stash_index] = PathOramBlock::dummy();
                } else {
                    self.blocks[stash_index] =
                        self.blocks[stash_index + (Z * ((height + 1) as usize))];
                    self.blocks[stash_index + (Z * ((height + 1) as usize))] =
                        PathOramBlock::dummy();
                }
            }

            physical_memory.write_db(position.node_on_path(depth, height) as usize, new_bucket);
        }

        // Clean up self
        self.high_water_mark = num_blocks_retained;
    }

    fn access<F: Fn(&V) -> V>(
        &mut self,
        address: crate::Address,
        new_position: super::TreeIndex,
        value_callback: F,
    ) -> V {
        let mut result: V = V::default();

        for block in &mut self.blocks {
            debug_assert_ne!(address, PathOramBlock::<V>::DUMMY_ADDRESS);

            let is_requested_index = block.address.ct_eq(&address);

            // Read current value of target block into `result``.
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

    fn occupancy(&self) -> StashSize {
        self.high_water_mark as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        database::SimpleDatabase,
        path_oram::{bucket::Bucket, path_oram_block::PathOramBlock, stash::Stash},
        Oram,
    };
    use rand::{rngs::StdRng, SeedableRng};

    use super::BitonicStash;

    #[test]
    fn play_around_with_oblivious_stash() {
        let mut s = BitonicStash::new(32);
        dbg!(&s);
        let block = PathOramBlock {
            value: 42,
            address: 0,
            position: 0b100,
        };
        dbg!(block.is_dummy());
        dbg!(block.ct_is_dummy());
        s.add_block(block);
        s.add_block(block);
        s.add_block(block);
        s.add_block(block);
        s.add_block(block);
        dbg!(&s);

        let mut rng = StdRng::seed_from_u64(0);
        let mut memory = SimpleDatabase::<Bucket<u64, 4>>::new(8, &mut rng);

        s.write_to_path(&mut memory, 2, 7);

        dbg!(&s);
        dbg!(&memory);
    }
}
