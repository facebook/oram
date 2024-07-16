// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Contains an abstract implementation of Path ORAM that is generic over its stash and position map data structures.

use super::{
    bucket::Bucket, position_map::PositionMap, stash::Stash, tree_index::CompleteBinaryTreeIndex,
    PathOramBlock, TreeHeight, TreeIndex, MAXIMUM_TREE_HEIGHT,
};
use crate::{
    database::{CountAccessesDatabase, Database},
    path_oram::address_oram_block::AddressOramBlock,
    utils::{
        invert_permutation_oblivious, random_permutation_of_0_through_n_exclusive, to_usize_vec,
    },
    Address, BlockSize, BucketSize, Oram, OramBlock,
};
use rand::{CryptoRng, Rng};
use std::mem;

/// A Path ORAM which is generic over its stash and position map data structures.
#[derive(Debug)]
pub struct GenericPathOram<
    V: OramBlock,
    const Z: BucketSize,
    const AB: BlockSize,
    P: PositionMap<AB>,
    S: Stash<V>,
> {
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
        const AB: BlockSize,
        P: PositionMap<AB> + std::fmt::Debug,
        S: Stash<V> + std::fmt::Debug,
    > Oram<V> for GenericPathOram<V, Z, AB, P, S>
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
        assert_ne!(new_position, 0);
        let position = self.position_map.write(address, new_position, rng);
        assert_ne!(position, 0);
        assert!(position.is_leaf(self.height));

        self.stash
            .read_from_path(&mut self.physical_memory, position);

        // Scan the stash for the target block, read its value into `result`,
        // and overwrite its position (and possibly its value).
        let result = self.stash.access(address, new_position, callback);

        // Evict blocks from the stash into the path that was just read,
        // replacing them with dummy blocks.
        self.stash
            .write_to_path(&mut self.physical_memory, position);

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

        let path_size = Z * (height as usize + 1);
        let stash = S::new(path_size, 128 - path_size);

        // physical_memory holds `block_capacity` buckets, each storing up to Z blocks.
        // The number of leaves is `block_capacity` / 2, which the original Path ORAM paper's experiments
        // found was sufficient to keep the stash size small with high probability.
        let mut physical_memory: CountAccessesDatabase<Bucket<V, Z>> =
            Database::new(number_of_nodes);

        // The rest of this function initializes the logical memory to contain default values at every address.
        // This is done by (1) initializing the position map with fresh random leaf identifiers,
        // and (2) writing blocks to the physical memory with the appropriate positions, and default values.

        let mut position_map = P::new(block_capacity, rng);

        let slot_indices_to_addresses =
            random_permutation_of_0_through_n_exclusive(block_capacity as u64, rng);
        let addresses_to_slot_indices = invert_permutation_oblivious(&slot_indices_to_addresses);
        let slot_indices_to_addresses = to_usize_vec(slot_indices_to_addresses);
        let mut addresses_to_slot_indices = to_usize_vec(addresses_to_slot_indices);

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
                    address: slot_indices_to_addresses[address_index],
                    position: leaf_index as TreeIndex,
                };
            }

            // Write the leaf bucket back to physical memory.
            physical_memory.write_db(leaf_index, bucket_to_write);
        }

        // The address block size might not divide the block capacity.
        // If it doesn't, we will have one block that contains dummy values.
        let mut num_blocks = block_capacity / AB;
        if block_capacity % AB > 0 {
            num_blocks += 1;
            addresses_to_slot_indices.resize(block_capacity + AB, 0);
        }

        for block_index in 0..num_blocks {
            let mut data = [0; AB];
            for i in 0..AB {
                data[i] = (first_leaf_index + addresses_to_slot_indices[block_index * AB + i] / 2)
                    as TreeIndex;
            }
            let block = AddressOramBlock::<AB> { data };
            position_map.write_position_block(block_index * AB, block, rng);
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
