// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A naive, insecure `Stash` implemented with a dynamically resizing Vec.

use super::{
    bucket::Bucket,
    path_oram_block::PathOramBlock,
    stash::{Stash, StashSize},
    tree_index::CompleteBinaryTreeIndex,
    TreeHeight, TreeIndex,
};
use crate::{database::Database, Address, BucketSize, OramBlock};
use std::ops::BitAnd;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

#[derive(Debug)]
/// A simple, non-oblivious stash implemented with a Vec.
pub struct VecStash<V: OramBlock>(Vec<PathOramBlock<V>>);

impl<V: OramBlock> VecStash<V> {
    fn write_stash_into_bucket<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        memory: &mut T,
        height: TreeHeight,
        depth: TreeHeight,
        position: TreeIndex,
    ) {
        let bucket_address = position.node_on_path(depth, height);

        let mut new_bucket: Bucket<V, Z> = Bucket::default();

        for slot_number in 0..Z {
            let slot = &mut new_bucket.blocks[slot_number];
            self.write_stash_into_slot(height, depth, position, slot);
        }

        memory.write_db(bucket_address as usize, new_bucket);
    }

    fn write_stash_into_slot(
        &mut self,
        height: TreeHeight,
        depth: TreeHeight,
        position: TreeIndex,
        slot: &mut PathOramBlock<V>,
    ) {
        let mut slot_already_written: Choice = 0.into();

        // LEAK: The time taken by this loop leaks the size of the stash.
        for stash_index in (0..self.0.len()).rev() {
            let stashed_block: &mut PathOramBlock<V> = &mut self.0[stash_index];

            let is_dummy = stashed_block.ct_is_dummy();

            // Compute whether the stashed block can be placed in the given slot.
            // The result of this computation is arbitrary if the stashed block is a dummy block.
            let mut stashed_block_position = stashed_block.position;
            let arbitrary_leaf = 2u64.pow(height);
            stashed_block_position.conditional_assign(&arbitrary_leaf, is_dummy);
            let stashed_block_assigned_bucket = stashed_block_position.node_on_path(depth, height);
            let slot_bucket = position.node_on_path(depth, height);
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
}

impl<V: OramBlock> Stash<V> for VecStash<V> {
    fn new(capacity: StashSize) -> Self {
        Self(vec![PathOramBlock::<V>::dummy(); capacity])
    }

    fn add_block(&mut self, block: PathOramBlock<V>) {
        self.0.push(block);
    }

    fn write_to_path<const Z: BucketSize, T: Database<Bucket<V, Z>>>(
        &mut self,
        physical_memory: &mut T,
        height: TreeHeight,
        position: TreeIndex,
    ) {
        for depth in (0..=height).rev() {
            self.write_stash_into_bucket(physical_memory, height, depth, position);
        }

        for stash_index in (0..self.0.len()).rev() {
            if self.0[stash_index].ct_is_dummy().into() {
                self.0.remove(stash_index);
                continue;
            }
        }
    }

    fn access<F: Fn(&V) -> V>(
        &mut self,
        address: Address,
        new_position: TreeIndex,
        value_callback: F,
    ) -> V {
        let mut result: V = V::default();

        // LEAK: The time taken by this loop leaks the size of the stash
        for block in &mut self.0 {
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
        let mut stash_occupancy = 0;
        for block in &self.0 {
            if !block.is_dummy() {
                stash_occupancy += 1;
            }
        }
        stash_occupancy
    }
}
