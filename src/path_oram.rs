// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Path ORAM.

use crate::{BlockValue, Database, IndexType, SimpleDatabase, ORAM};
use rand::{thread_rng, Rng};
use std::ops::BitAnd;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

const MAXIMUM_TREE_HEIGHT: u32 = 63;
const DEFAULT_BLOCKS_PER_BUCKET: usize = 4;

#[derive(Clone, Copy, Default)]
struct PathORAMBlock<const B: usize> {
    value: BlockValue<B>,
    address: IndexType,
    position: u64,
}

impl<const B: usize> PathORAMBlock<B> {
    const DUMMY_ADDRESS: IndexType = 0;
}

impl<const B: usize> ConditionallySelectable for PathORAMBlock<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let value = BlockValue::conditional_select(&a.value, &b.value, choice);
        let address =
            u64::conditional_select(&(a.address as u64), &(b.address as u64), choice) as usize;
        let position = u64::conditional_select(&a.position, &b.position, choice);
        PathORAMBlock::<B> {
            value,
            address,
            position,
        }
    }
}

/// A simple, insecure implementation of Path ORAM
/// whose stash is just a Vec of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// this implementation would only leak the size of the stash at each access
/// via a timing side-channel.
/// (Such leakage would likely still be unacceptable.)
struct NonrecursiveClientStashPathORAM<const B: usize, const Z: usize> {
    physical_memory: SimpleDatabase<Bucket<B, Z>>,
    stash: Vec<PathORAMBlock<B>>,
    position_map: SimpleDatabase<CompleteBinaryTreeIndex>,
    height: u32,
}

impl<const B: usize, const Z: usize> ORAM<B> for NonrecursiveClientStashPathORAM<B, Z> {
    fn access(
        &mut self,
        index: crate::IndexType,
        optional_new_value: subtle::CtOption<BlockValue<B>>,
    ) -> BlockValue<B> {
        let leaf = self.position_map.read(index);

        // Read all blocks on the relevant path into the stash
        for depth in 0..self.height {
            let node = CompleteBinaryTreeIndex::node_on_path(leaf, depth);
            let bucket = self.physical_memory.read(node.index as usize);
            for block in bucket.blocks {
                self.stash.push(block);
            }
        }

        let mut result: BlockValue<B> = BlockValue::default();

        // Linearly scan stash to read and potentially update target block
        for block in &mut self.stash {
            let is_requested_index = block.address.ct_eq(&index);
            result.conditional_assign(&block.value, is_requested_index);

            let oram_operation_is_write = optional_new_value.is_some();
            let should_write = is_requested_index.bitand(oram_operation_is_write);
            let value_to_write: BlockValue<B> =
                optional_new_value.unwrap_or_else(BlockValue::default);

            block
                .value
                .conditional_assign(&value_to_write, should_write);
        }

        // Working from leaves to root, write stash back into path, obliviously but inefficiently.
        for depth in (0..self.height).rev() {
            let bucket_address: CompleteBinaryTreeIndex =
                CompleteBinaryTreeIndex::node_on_path(leaf, depth);
            let mut new_bucket: Bucket<B, Z> = Bucket::default();

            for slot_index in 0..DEFAULT_BLOCKS_PER_BUCKET {
                // For each slot on the relevant path...
                let slot = &mut new_bucket.blocks[slot_index];

                // Linearly scan the stash for a block that can be written into that slot.
                let mut written: Choice = 0.into();

                for stash_index in 0..self.stash.len() {
                    let stashed_block = &mut self.stash[stash_index];

                    let position = stashed_block.position;
                    let position_index = CompleteBinaryTreeIndex::new(self.height, position);
                    let assigned_bucket_address =
                        CompleteBinaryTreeIndex::node_on_path(position_index, depth);
                    let position_matches =
                        bucket_address.index.ct_eq(&assigned_bucket_address.index);
                    let should_write = position_matches.bitand(!written);

                    // If found, write the slot and overwrite the stash block with a dummy block.
                    slot.conditional_assign(stashed_block, should_write);
                    stashed_block.conditional_assign(&PathORAMBlock::default(), should_write);
                    written.conditional_assign(&(1.into()), should_write);
                }
            }
            self.physical_memory
                .write(bucket_address.index as usize, new_bucket);

            // Remove dummy blocks from the stash.
            for stash_index in (0..self.stash.len()).rev() {
                let stash_block = &mut self.stash[stash_index];
                if stash_block.address == PathORAMBlock::<B>::DUMMY_ADDRESS {
                    self.stash.remove(stash_index);
                }
            }
        }

        result
    }

    fn new(N: usize) -> Self {
        assert!(N.is_power_of_two());

        let number_of_nodes = N;
        let physical_memory = SimpleDatabase::new(number_of_nodes);
        // physical_memory holds N buckets, each storing up to Z blocks.
        // The capacity of physical_memory in blocks is thus Z * N.
        // The number of leaves is N / 2, which the original Path ORAM paper's experiments found was sufficient.

        let stash = Vec::new();

        let mut position_map = SimpleDatabase::new(N);
        let height: u32 = N.ilog2();
        for i in 0..position_map.capacity() {
            position_map.write(i, CompleteBinaryTreeIndex::random_leaf(height));
        }

        Self {
            physical_memory,
            stash,
            position_map,
            height,
        }
    }

    fn block_size(&self) -> crate::IndexType {
        B
    }

    fn block_capacity(&self) -> crate::IndexType {
        self.physical_memory.capacity()
    }
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
/// A Path ORAM bucket.
pub struct Bucket<const B: usize, const Z: usize> {
    // Should use GenericArray?
    blocks: [PathORAMBlock<B>; Z],
}

impl<const B: usize, const Z: usize> Bucket<B, Z> {
    fn block_size(&self) -> usize {
        B
    }
}

impl<const B: usize, const Z: usize> Default for Bucket<B, Z> {
    fn default() -> Self {
        Self {
            blocks: [PathORAMBlock::<B>::default(); Z],
        }
    }
}

/// Represents the array index of an element in a binary tree laid out as an array.
#[derive(Clone, Copy, Debug)]
pub struct CompleteBinaryTreeIndex {
    /// The height of the tree in which this `CompleteBinaryTreeIndex` represents a node.
    tree_height: u32,
    /// The index of the node represented by this `CompleteBinaryTreeIndex`.
    index: u64,
    /// The depth of the node represented by this `CompleteBinaryTreeIndex`.
    /// The root has depth 0 and the leaves have depth `tree_height`.
    depth: u32,
}

impl CompleteBinaryTreeIndex {
    fn new(tree_height: u32, index: u64) -> Self {
        assert!(index != 0);
        assert!(tree_height <= MAXIMUM_TREE_HEIGHT);
        let tree_size = 2u64.pow(tree_height + 1);
        assert!(index < tree_size);

        let leading_zeroes = index.leading_zeros();
        let index_bitlength = 64;
        let depth = index_bitlength - leading_zeroes - 1;
        CompleteBinaryTreeIndex {
            tree_height,
            index,
            depth,
        }
    }

    fn node_on_path(leaf: CompleteBinaryTreeIndex, depth: u32) -> CompleteBinaryTreeIndex {
        assert!(leaf.is_leaf());
        let leaf_index = leaf.index;
        let height = leaf.tree_height;
        let shift = leaf.tree_height - depth;
        let node_index = leaf_index >> shift;
        CompleteBinaryTreeIndex::new(height, node_index)
    }

    fn random_leaf(tree_height: u32) -> Self {
        let random_index =
            2u64.pow(tree_height - 1) + thread_rng().gen_range(0..2u64.pow(tree_height - 1));
        CompleteBinaryTreeIndex::new(tree_height, random_index)
    }

    fn get_depth(&self) -> u32 {
        self.depth
    }

    fn is_root(&self) -> bool {
        self.index == 1
    }

    fn is_leaf(&self) -> bool {
        self.depth == self.tree_height
    }

    fn parent(&self) -> Option<Self> {
        if self.is_root() {
            None
        } else {
            Some(CompleteBinaryTreeIndex::new(
                self.tree_height,
                self.index >> 1,
            ))
        }
    }

    fn left_child(&self) -> Option<Self> {
        if self.is_leaf() {
            None
        } else {
            Some(CompleteBinaryTreeIndex::new(
                self.tree_height,
                self.index << 1,
            ))
        }
    }

    fn right_child(&self) -> Option<Self> {
        if self.is_leaf() {
            None
        } else {
            Some(CompleteBinaryTreeIndex::new(
                self.tree_height,
                (self.index << 1) + 1,
            ))
        }
    }
}

impl Default for CompleteBinaryTreeIndex {
    fn default() -> Self {
        CompleteBinaryTreeIndex::new(1, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::{CompleteBinaryTreeIndex, DEFAULT_BLOCKS_PER_BUCKET};
    use crate::{path_oram::NonrecursiveClientStashPathORAM, ORAM};

    #[test]
    fn play_around_with_index() {
        let index011 = CompleteBinaryTreeIndex::new(3, 0b011);
        let parent = index011.parent().unwrap();
        let left_child = index011.left_child().unwrap();
        let right_child = index011.right_child().unwrap();
        assert_eq!(parent.index, 0b01);
        assert_eq!(left_child.index, 0b110);
        assert_eq!(right_child.index, 0b0111);
        assert_eq!(index011.depth, 1);
        assert_eq!(parent.depth, 0);
        assert_eq!(left_child.depth, 2);
        assert_eq!(right_child.depth, 2);

        println!("{:?}", index011);
        println!("{:?}", parent);
        println!("{:?}", left_child);
        println!("{:?}", right_child);
    }

    #[test]
    fn play_around_with_path_oram() {
        let mut oram: NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET> =
            NonrecursiveClientStashPathORAM::new(64);
        // let oram: PathORAM<
        //     SimpleDatabase<Bucket<BlockValue<64>, DEFAULT_BLOCKS_PER_BUCKET>>,
        //     SimpleDatabase<CompleteBinaryTreeIndex>,
        //     DEFAULT_BLOCKS_PER_BUCKET,
        // > = PathORAM::new(64);
        println!("{:?}", oram.read(0));
    }
}
