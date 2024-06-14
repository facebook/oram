// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Path ORAM.

use crate::{BlockValue, Database, IndexType, SimpleDatabase, ORAM};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{mem::size_of_val, ops::BitAnd};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

const MAXIMUM_TREE_HEIGHT: u32 = 63;
const DEFAULT_BLOCKS_PER_BUCKET: usize = 4;

/// A simple, insecure implementation of Path ORAM
/// whose stash is just a Vec of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// this implementation would only leak the size of the stash at each access
/// via a timing side-channel.
/// (Such leakage would likely still be unacceptable.)
#[derive(Debug)]
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
        let new_position = CompleteBinaryTreeIndex::random_leaf(self.height);
        self.position_map.write(index, new_position);

        // Read all blocks on the relevant path into the stash
        for depth in 0..=self.height {
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
            block
                .position
                .conditional_assign(&new_position.index, should_write);
        }

        // Working from leaves to root, write stash back into path, obliviously but inefficiently.
        for depth in (0..=self.height).rev() {
            let bucket_address: CompleteBinaryTreeIndex =
                CompleteBinaryTreeIndex::node_on_path(leaf, depth);
            let mut new_bucket: Bucket<B, Z> = Bucket::default();

            for slot_index in 0..DEFAULT_BLOCKS_PER_BUCKET {
                // For each slot on the relevant path...
                let slot = &mut new_bucket.blocks[slot_index];

                // Linearly scan the stash for a block that can be written into that slot,
                // removing dummy blocks as we go.
                let mut written: Choice = 0.into();

                for stash_index in (0..self.stash.len()).rev() {
                    let stashed_block = &mut self.stash[stash_index];
                    if stashed_block.address == PathORAMBlock::<B>::DUMMY_ADDRESS {
                        self.stash.remove(stash_index);
                    } else {
                        let position = stashed_block.position;
                        let position_index = CompleteBinaryTreeIndex::new(self.height, position);
                        let assigned_bucket_address =
                            CompleteBinaryTreeIndex::node_on_path(position_index, depth);
                        let position_matches =
                            bucket_address.index.ct_eq(&assigned_bucket_address.index);
                        let should_write = position_matches.bitand(!written);

                        // If found, write the slot and overwrite the stash block with a dummy block.
                        slot.conditional_assign(stashed_block, should_write);
                        stashed_block.conditional_assign(&PathORAMBlock::dummy(), should_write);
                        written.conditional_assign(&(1.into()), should_write);
                    }
                }
            }
            self.physical_memory
                .write(bucket_address.index as usize, new_bucket);
        }

        result
    }

    fn new(block_capacity: usize) -> Self {
        assert!(block_capacity.is_power_of_two());

        let number_of_nodes = block_capacity;

        // physical_memory holds N buckets, each storing up to Z blocks.
        // The capacity of physical_memory in blocks is thus Z * N.
        // The number of leaves is N / 2, which the original Path ORAM paper's experiments found was sufficient.
        let mut physical_memory = SimpleDatabase::new(number_of_nodes);

        let stash = Vec::new();

        let height: u32 = block_capacity.ilog2() - 1;

        // We initialize the physical memory with blocks containing 0
        let permuted_addresses = 0..block_capacity;
        let mut permuted_addresses = Vec::from_iter(permuted_addresses);
        let permuted_addresses = permuted_addresses.as_mut_slice();
        permuted_addresses.shuffle(&mut thread_rng());

        let mut position_map = SimpleDatabase::new(block_capacity);
        // for i in 0..position_map.capacity() {
        //     position_map.write(i, CompleteBinaryTreeIndex::random_leaf(height));
        // }

        let first_leaf_index = 2u64.pow(height) as usize;
        let last_leaf_index = (2 * first_leaf_index) - 1;

        let addresses_per_leaf = 2;
        for leaf_index in first_leaf_index..=last_leaf_index {
            let mut bucket_to_write = Bucket::<B, Z>::default();
            for block_index in 0..addresses_per_leaf {
                let address_index = (leaf_index - first_leaf_index) * 2 + block_index;
                bucket_to_write.blocks[block_index] = PathORAMBlock::<B> {
                    value: BlockValue::<B>::default(),
                    address: permuted_addresses[address_index],
                    position: leaf_index as u64,
                };
                position_map.write(permuted_addresses[address_index], CompleteBinaryTreeIndex::new(height, leaf_index as u64));
            }
            physical_memory.write(leaf_index, bucket_to_write);
        }

        for leaf_index in 1..block_capacity {
            for block in physical_memory.read(leaf_index).blocks {
                if block.address != PathORAMBlock::<B>::DUMMY_ADDRESS {
                    assert_eq!(block.position, position_map.read(block.address).index);
                    println!("{}, {}", block.position, position_map.read(block.address).index);
                }
            }
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

#[derive(Clone, Copy, Default, Debug)]
struct PathORAMBlock<const B: usize> {
    value: BlockValue<B>,
    address: IndexType,
    position: u64,
}

impl<const B: usize> PathORAMBlock<B> {
    const DUMMY_ADDRESS: IndexType = IndexType::MAX;
    const DUMMY_POSITION: u64 = u64::MAX;

    fn dummy() -> Self {
        Self {
            value: BlockValue::default(),
            address: Self::DUMMY_ADDRESS,
            position: Self::DUMMY_POSITION,
        }
    }
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

#[repr(align(4096))]
#[derive(Clone, Copy, Debug)]
/// A Path ORAM bucket.
pub struct Bucket<const B: usize, const Z: usize> {
    blocks: [PathORAMBlock<B>; Z],
}

impl<const B: usize, const Z: usize> Default for Bucket<B, Z> {
    fn default() -> Self {
        Self {
            blocks: [PathORAMBlock::<B>::dummy(); Z],
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
        assert_ne!(index, 0);
        assert!(tree_height <= MAXIMUM_TREE_HEIGHT);
        let tree_size = 2u64.pow(tree_height + 1);
        assert!(index < tree_size);

        let leading_zeroes = index.leading_zeros();
        let index_bitlength = 8 * (size_of_val(&index) as u32);
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
        let random_index = 2u64.pow(tree_height) + thread_rng().gen_range(0..2u64.pow(tree_height));
        let result = CompleteBinaryTreeIndex::new(tree_height, random_index);
        assert!(result.is_leaf());
        result
    }

    fn is_leaf(&self) -> bool {
        self.depth == self.tree_height
    }
}

impl Default for CompleteBinaryTreeIndex {
    fn default() -> Self {
        CompleteBinaryTreeIndex::new(1, 1)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_correctness_random_workload;

    use crate::{path_oram::NonrecursiveClientStashPathORAM, ORAM};

    #[test]
    fn scratch() {
        let mut oram: NonrecursiveClientStashPathORAM<1, DEFAULT_BLOCKS_PER_BUCKET> = NonrecursiveClientStashPathORAM::new(2);
        dbg!(&oram);
        dbg!(oram.read(0));
        dbg!(oram.read(1));
        dbg!(oram.write(0, BlockValue::from_byte_array([37u8; 1])));
        dbg!(&oram);
    }

    #[test]
    fn play_around_with_path_oram() {
        let mut oram: NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET> =
            NonrecursiveClientStashPathORAM::new(64);
        println!("{:?}", oram.read(1));
    }

    #[test]
    fn test_correctness_random_workload_1_4_10000() {
        test_correctness_random_workload::<1, NonrecursiveClientStashPathORAM<1, DEFAULT_BLOCKS_PER_BUCKET>>(4, 10000);
    }

    // #[test]
    // fn test_correctness_random_workload_2_4_10000() {
    //     test_correctness_random_workload::<2, NonrecursiveClientStashPathORAM<2, DEFAULT_BLOCKS_PER_BUCKET>>(4, 10000);
    // }

    // #[test]
    // fn test_correctness_random_workload_1_64_10000() {
    //     test_correctness_random_workload::<1, NonrecursiveClientStashPathORAM<1, DEFAULT_BLOCKS_PER_BUCKET>>(64, 10000);
    // }

    // #[test]
    // fn test_correctness_random_workload_64_1_10000() {
    //     test_correctness_random_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(1, 10000);
    // }

    #[test]
    fn test_correctness_random_workload_4_4_10000() {
        test_correctness_random_workload::<4, NonrecursiveClientStashPathORAM<4, DEFAULT_BLOCKS_PER_BUCKET>>(4, 10000);
    }

    // #[test]
    // fn test_correctness_random_workload_64_64_10000() {
    //     test_correctness_random_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(64, 10000);
    // }

    // #[test]
    // fn test_correctness_random_workload_64_256_10000() {
    //     test_correctness_random_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(256, 10000);
    // }

    // #[test]
    // fn test_correctness_random_workload_4096_64_1000() {
    //     test_correctness_random_workload::<4096, NonrecursiveClientStashPathORAM<4096, DEFAULT_BLOCKS_PER_BUCKET>>(200, 1000);
    // }

    // #[test]
    // fn test_correctness_random_workload_4096_256_1000() {
    //     test_correctness_random_workload::<4096, NonrecursiveClientStashPathORAM<4096, DEFAULT_BLOCKS_PER_BUCKET>>(256, 1000);
    // }

    // #[test]
    // fn test_correctness_linear_workload_1_64_100() {
    //     test_correctness_linear_workload::<1, NonrecursiveClientStashPathORAM<1, DEFAULT_BLOCKS_PER_BUCKET>>(64, 100);
    // }

    // #[test]
    // fn test_correctness_linear_workload_64_1_100() {
    //     test_correctness_linear_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(1, 100);
    // }

    // #[test]
    // fn test_correctness_linear_workload_64_64_100() {
    //     test_correctness_linear_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(64, 100);
    // }

    // #[test]
    // fn test_correctness_linear_workload_64_256_100() {
    //     test_correctness_linear_workload::<64, NonrecursiveClientStashPathORAM<64, DEFAULT_BLOCKS_PER_BUCKET>>(256, 100);
    // }

    // #[test]
    // fn test_correctness_linear_workload_4096_64_10() {
    //     test_correctness_linear_workload::<4096, NonrecursiveClientStashPathORAM<4096, DEFAULT_BLOCKS_PER_BUCKET>>(64, 10);
    // }

    // #[test]
    // fn test_correctness_linear_workload_4096_256_2() {
    //     test_correctness_linear_workload::<4096, NonrecursiveClientStashPathORAM<4096, DEFAULT_BLOCKS_PER_BUCKET>>(256, 2);
    // }
}