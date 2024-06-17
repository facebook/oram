// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and (non-recursive) position map.

use crate::{
    BlockSizeType, BlockValue, BucketSizeType, CountAccessesDatabase, Database, IndexType, Oram, TreeHeight, TreeIndex, DEFAULT_BLOCKS_PER_BUCKET, MAXIMUM_TREE_HEIGHT
};
use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::{mem::size_of, ops::BitAnd};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

/// (!) This is a development stepping stone, not a finished implementation. (!)
/// A simple, insecure implementation of Path ORAM
/// whose stash is just a `Vec` of blocks that is accessed non-obliviously.
/// In our scenario where the stash is in untrusted storage,
/// this is of course completely unacceptable.
/// In the scenario where the stash is in trusted client storage,
/// this implementation would only leak the size of the stash at each access
/// via a timing side-channel.
/// (Such leakage would likely still be unacceptable.)
/// The leakage will be addressed by more sophisticated stash access routines
/// in one of the next few iterations.
#[derive(Debug)]
pub struct SimpleInsecurePathOram<const B: BlockSizeType, const Z: BucketSizeType, R: Rng> {
    /// Again, making the ORAM untrusted storage `physical_memory` public for now, for benchmarking purposes.
    pub physical_memory: CountAccessesDatabase<Bucket<B, Z>>,
    stash: Vec<PathOramBlock<B>>,
    position_map: CountAccessesDatabase<TreeIndex>,
    height: TreeHeight,
    rng: R,
}

impl<const B: BlockSizeType, const Z: BucketSizeType, R: Rng> Oram<B, R> for SimpleInsecurePathOram<B, Z, R> {
    fn access(
        &mut self,
        index: crate::IndexType,
        optional_new_value: subtle::CtOption<BlockValue<B>>,
    ) -> BlockValue<B> {
        let leaf = self.position_map.read(index);
        let new_position = CompleteBinaryTreeIndex::random_leaf(self.height, &mut self.rng);
        self.position_map.write(index, new_position);

        // Read all blocks on the relevant path into the stash
        for depth in 0..=self.height {
            // let node = CompleteBinaryTreeIndex::node_on_path(leaf, depth);
            let node = leaf.node_on_path(depth, self.height);
            let bucket = self.physical_memory.read(node as usize);
            for block in bucket.blocks {
                self.stash.push(block);
            }
        }

        let mut result: BlockValue<B> = BlockValue::default();
        let value_to_write: BlockValue<B> = optional_new_value.unwrap_or_else(BlockValue::default);
        let oram_operation_is_write = optional_new_value.is_some();

        // Linearly scan stash to read and potentially update target block
        for block in &mut self.stash {
            let is_requested_index = block.address.ct_eq(&index);

            // Read current value of target block into result
            result.conditional_assign(&block.value, is_requested_index);

            // Write new position into target block
            block
                .position
                .conditional_assign(&new_position, is_requested_index);

            let should_write = is_requested_index.bitand(oram_operation_is_write);
            // Write new value and position into target block in case of write
            block
                .value
                .conditional_assign(&value_to_write, should_write);
        }

        // Working from leaves to root, write stash back into path, obliviously but inefficiently.
        for depth in (0..=self.height).rev() {
            // let bucket_address: CompleteBinaryTreeIndex =
            //     CompleteBinaryTreeIndex::node_on_path(leaf, depth);
            let bucket_address = leaf.node_on_path(depth, self.height);
            let mut new_bucket: Bucket<B, Z> = Bucket::default();

            for slot_index in 0..DEFAULT_BLOCKS_PER_BUCKET {
                // For each slot on the relevant path...
                let slot = &mut new_bucket.blocks[slot_index];

                // Linearly scan the stash for a block that can be written into that slot,
                // removing dummy blocks as we go.
                let mut written: Choice = 0.into();

                for stash_index in (0..self.stash.len()).rev() {
                    let stashed_block = &mut self.stash[stash_index];
                    if stashed_block.address == PathOramBlock::<B>::DUMMY_ADDRESS {
                        self.stash.remove(stash_index);
                    } else {
                        let position = stashed_block.position;
                        // let position_index = CompleteBinaryTreeIndex::new(self.height, position);
                        let assigned_bucket_address = position.node_on_path(depth, self.height);
                        let position_matches =
                            bucket_address.ct_eq(&assigned_bucket_address);
                        let should_write = position_matches.bitand(!written);

                        // If found, write the slot and overwrite the stash block with a dummy block.
                        slot.conditional_assign(stashed_block, should_write);
                        stashed_block.conditional_assign(&PathOramBlock::dummy(), should_write);
                        written.conditional_assign(&(1.into()), should_write);
                    }
                }
            }
            self.physical_memory
                .write(bucket_address as usize, new_bucket);
        }

        result
    }

    fn new(block_capacity: usize, mut rng: R) -> Self {
        assert!(block_capacity.is_power_of_two(), "{}", block_capacity);
        assert!(block_capacity > 1);

        let number_of_nodes = block_capacity;

        // physical_memory holds N buckets, each storing up to Z blocks.
        // The capacity of physical_memory in blocks is thus Z * N.
        // The number of leaves is N / 2, which the original Path ORAM paper's experiments found was sufficient.
        let mut physical_memory = CountAccessesDatabase::new(number_of_nodes);

        let stash = Vec::new();

        let height = block_capacity.ilog2() - 1;
        assert!(height <= MAXIMUM_TREE_HEIGHT);

        // We initialize the physical memory with blocks containing 0
        let permuted_addresses = 0..block_capacity;
        let mut permuted_addresses = Vec::from_iter(permuted_addresses);
        let permuted_addresses = permuted_addresses.as_mut_slice();
        permuted_addresses.shuffle(&mut rng);

        let mut position_map = CountAccessesDatabase::new(block_capacity);

        let first_leaf_index = 2u64.pow(height) as usize;
        let last_leaf_index = (2 * first_leaf_index) - 1;

        let addresses_per_leaf = 2;
        for leaf_index in first_leaf_index..=last_leaf_index {
            let mut bucket_to_write = Bucket::<B, Z>::default();
            for block_index in 0..addresses_per_leaf {
                let address_index = (leaf_index - first_leaf_index) * 2 + block_index;
                bucket_to_write.blocks[block_index] = PathOramBlock::<B> {
                    value: BlockValue::<B>::default(),
                    address: permuted_addresses[address_index],
                    position: leaf_index as TreeIndex,
                };
                position_map.write(
                    permuted_addresses[address_index], leaf_index as TreeIndex,
                );
            }
            physical_memory.write(leaf_index, bucket_to_write);
        }

        Self {
            physical_memory,
            stash,
            position_map,
            height,
            rng,
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
struct PathOramBlock<const B: BlockSizeType> {
    value: BlockValue<B>,
    address: IndexType,
    position: TreeIndex,
}

impl<const B: BlockSizeType> PathOramBlock<B> {
    const DUMMY_ADDRESS: IndexType = IndexType::MAX;
    const DUMMY_POSITION: TreeIndex = u64::MAX;

    fn dummy() -> Self {
        Self {
            value: BlockValue::default(),
            address: Self::DUMMY_ADDRESS,
            position: Self::DUMMY_POSITION,
        }
    }
}

impl<const B: BlockSizeType> ConditionallySelectable for PathOramBlock<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let value = BlockValue::conditional_select(&a.value, &b.value, choice);
        let address =
            u64::conditional_select(&(a.address as u64), &(b.address as u64), choice) as usize;
        let position = u64::conditional_select(&a.position, &b.position, choice);
        PathOramBlock::<B> {
            value,
            address,
            position,
        }
    }
}

#[repr(align(4096))]
#[derive(Clone, Copy, Debug)]
/// A Path ORAM bucket.
pub struct Bucket<const B: BlockSizeType, const Z: BucketSizeType> {
    blocks: [PathOramBlock<B>; Z],
}

impl<const B: BlockSizeType, const Z: BucketSizeType> Default for Bucket<B, Z> {
    fn default() -> Self {
        Self {
            blocks: [PathOramBlock::<B>::dummy(); Z],
        }
    }
}

trait CompleteBinaryTreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self;
    fn random_leaf<R: Rng>(tree_height: TreeHeight, rng: R) -> Self;
    fn depth(&self) -> TreeHeight;
    fn is_leaf(&self, height: TreeHeight) -> bool;
}

impl CompleteBinaryTreeIndex for TreeIndex {
    fn node_on_path(&self, depth: TreeHeight, height: TreeHeight) -> Self {
        assert!(self.is_leaf(height));
        let shift = height - depth;
        let node_index = self >> shift;
        node_index
    }

    fn random_leaf<R: Rng>(tree_height: TreeHeight, mut rng: R) -> Self {
        2u64.pow(tree_height) + rng.gen_range(0..2u64.pow(tree_height))
    }

    fn depth(&self) -> TreeHeight {
        let leading_zeroes = self.leading_zeros();
        let index_bitlength = 8 * (size_of::<TreeIndex>() as TreeHeight);
        index_bitlength - leading_zeroes - 1
    }

    fn is_leaf(&self, height: TreeHeight) -> bool {
        self.depth() == height
    }
}

/// A type alias for a simple `SimpleInsecurePathOram` monomorphization.
pub type ConcreteSimpleInsecurePathOram<const B: BlockSizeType> =
    SimpleInsecurePathOram<B, DEFAULT_BLOCKS_PER_BUCKET, StdRng>;

#[cfg(test)]
mod tests {
    use crate::test_utils::{
        create_correctness_test, test_correctness_linear_workload, test_correctness_random_workload,
    };

    use super::ConcreteSimpleInsecurePathOram;
    create_correctness_test!(
        test_correctness_random_workload,
        ConcreteSimpleInsecurePathOram,
        64,
        256,
        10000
    );
    create_correctness_test!(
        test_correctness_random_workload,
        ConcreteSimpleInsecurePathOram,
        1,
        64,
        10000
    );
    create_correctness_test!(
        test_correctness_random_workload,
        ConcreteSimpleInsecurePathOram,
        64,
        64,
        10000
    );
    create_correctness_test!(
        test_correctness_random_workload,
        ConcreteSimpleInsecurePathOram,
        4096,
        64,
        1000
    );
    create_correctness_test!(
        test_correctness_random_workload,
        ConcreteSimpleInsecurePathOram,
        4096,
        256,
        1000
    );

    create_correctness_test!(
        test_correctness_linear_workload,
        ConcreteSimpleInsecurePathOram,
        64,
        256,
        100
    );
    create_correctness_test!(
        test_correctness_linear_workload,
        ConcreteSimpleInsecurePathOram,
        1,
        64,
        100
    );
    create_correctness_test!(
        test_correctness_linear_workload,
        ConcreteSimpleInsecurePathOram,
        64,
        64,
        100
    );
    create_correctness_test!(
        test_correctness_linear_workload,
        ConcreteSimpleInsecurePathOram,
        4096,
        64,
        10
    );
    create_correctness_test!(
        test_correctness_linear_workload,
        ConcreteSimpleInsecurePathOram,
        4096,
        256,
        2
    );
}
