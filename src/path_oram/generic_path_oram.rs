// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple, insecure implementation of Path ORAM with "client-side" stash and (non-recursive) position map.

use super::{
    bucket::Bucket, stash::Stash, tree_index::CompleteBinaryTreeIndex, PathOramBlock, TreeHeight,
    TreeIndex, MAXIMUM_TREE_HEIGHT,
};
use crate::{
    database::{CountAccessesDatabase, Database},
    utils::random_permutation_of_0_through_n_exclusive,
    Address, BucketSize, Oram, OramBlock,
};
use rand::{CryptoRng, Rng};
use std::mem;

/// A Path ORAM with generic position map and stash.
#[derive(Debug)]
pub struct GenericPathOram<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex>, S: Stash<V>> {
    // The fields below are not meant to be exposed to clients. They are public for benchmarking and testing purposes.
    /// The underlying untrusted memory that the ORAM is obliviously accessing on behalf of its client.
    pub physical_memory: CountAccessesDatabase<Bucket<V, Z>>,
    /// The Path ORAM stash.
    pub stash: S,
    /// The Path ORAM position map.
    pub position_map: P,
    /// The height of the Path ORAM tree data structure.
    pub height: TreeHeight,
}

impl<
        V: OramBlock,
        const Z: BucketSize,
        P: Oram<TreeIndex> + std::fmt::Debug,
        S: Stash<V> + std::fmt::Debug,
    > Oram<V> for GenericPathOram<V, Z, P, S>
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
        debug_assert!(position.is_leaf(self.height));

        // self.read_path_into_stash(position);
        for depth in 0..=self.height {
            let bucket_index: TreeIndex = position.node_on_path(depth, self.height);
            let bucket = self.physical_memory.read_db(bucket_index as Address);
            for block in bucket.blocks {
                self.stash.add_block(block);
            }
        }

        // LEAK: The next three lines leak information
        // about the state of the stash at the end of the access.

        // Scan the stash for the target block, read its value into `result`,
        // and overwrite its position (and possibly its value).
        // let result = self.access_stash(address, callback, new_position, rng);
        let result = self.stash.access(address, new_position, callback);

        // Evict blocks from the stash into the path that was just read,
        // replacing them with dummy blocks.
        // self.write_stash_into_tree_path(position);
        self.stash
            .write_to_path(&mut self.physical_memory, self.height, position);

        // Remove accumulated dummy blocks from the stash.
        // self.cleanup_stash();

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

        // let stash: Vec<PathOramBlock<V>> = Vec::new();
        let stash = S::new(64);

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory: CountAccessesDatabase<Bucket<V, Z>> =
            Database::new(number_of_nodes);

        // The rest of this function initializes the logical memory to contain default values at every address.
        // This is done by (1) initializing the position map with fresh random leaf identifiers,
        // and (2) writing blocks to the physical memory with the appropriate positions, and default values.

        let mut position_map = P::new(block_capacity, rng);

        let permuted_addresses = random_permutation_of_0_through_n_exclusive(block_capacity, rng);

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
