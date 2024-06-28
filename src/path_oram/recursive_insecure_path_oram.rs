// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An insecure implementation of Path ORAM with "client-side" stash and recursive position map.

use crate::{
    database::SimpleDatabase, linear_time_oram::LinearTimeOram, Address, BlockSize, BucketSize,
    Oram, OramBlock,
};

use super::{address_oram_block::AddressOramBlock, TreeIndex};

use log::debug;
use rand::{CryptoRng, RngCore};
use subtle::{ConditionallySelectable, ConstantTimeEq};

use super::simple_insecure_path_oram::VecPathOram;

/// A Path ORAM with a recursive position map and "client-side" stash.
pub type BlockOram<const AB: BlockSize, V, const Z: BucketSize> =
    VecPathOram<V, Z, AddressOram<AB, Z>>;

impl<const AB: BlockSize, V: OramBlock, const Z: BucketSize> BlockOram<AB, V, Z> {
    pub(crate) fn recursion_height(&self) -> usize {
        self.position_map.recursion_height()
    }
}

/// An `Oram` intended for use as a position map. `AB` is the block size in addresses.
#[derive(Debug)]
pub enum AddressOram<const AB: BlockSize, const Z: BucketSize> {
    /// A simple, linear-time `AddressOram`.
    Base(LinearTimeOram<SimpleDatabase<TreeIndex>>),
    /// A recursive `AddressOram` whose position map is a `VecPathOram`.
    Recursive(Box<VecPathOram<AddressOramBlock<AB>, Z, AddressOram<AB, Z>>>),
}

impl<const AB: BlockSize, const Z: BucketSize> AddressOram<AB, Z> {
    pub(crate) fn recursion_height(&self) -> usize {
        match self {
            AddressOram::Base(_) => 0,
            AddressOram::Recursive(inner) => 1 + inner.recursion_height(),
        }
    }

    fn address_of_block(address: Address) -> Address {
        assert!(AB.is_power_of_two());
        let block_address_bits = AB.ilog2();
        address >> block_address_bits
    }

    fn address_within_block(address: Address) -> Address {
        assert!(AB.is_power_of_two());
        let block_address_bits = AB.ilog2();
        let shift = (Address::BITS as usize) - (block_address_bits as usize);
        (address << shift) >> shift
    }
}

impl<const B: BlockSize, const Z: BucketSize> Oram<TreeIndex> for AddressOram<B, Z> {
    fn new<R: CryptoRng + RngCore>(number_of_addresses: Address, rng: &mut R) -> Self {
        debug!(
            "Oram::new -- AddressOram(B = {}, Z = {}, C = {})",
            B, Z, number_of_addresses
        );

        assert!(B >= 2);

        if number_of_addresses <= B {
            Self::Base(LinearTimeOram::new(number_of_addresses, rng))
        } else {
            assert!(number_of_addresses % B == 0);
            let block_capacity = number_of_addresses / B;
            Self::Recursive(Box::new(VecPathOram::new(block_capacity, rng)))
        }
    }

    fn block_capacity(&self) -> Address {
        match self {
            AddressOram::Base(linear_oram) => linear_oram.block_capacity(),
            AddressOram::Recursive(block_oram) => block_oram.block_capacity() * B,
        }
    }

    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> TreeIndex {
        log::debug!(
            "Level {} AddressORAM read: {}",
            self.recursion_height(),
            index
        );
        let callback = |x: &TreeIndex| *x;
        self.access(index, callback, rng)
    }

    /// Obliviously writes the value stored at `index`. Returns the value previously stored at `index`.
    fn write<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        new_value: TreeIndex,
        rng: &mut R,
    ) -> TreeIndex {
        log::debug!(
            "Level {} AddressORAM write: {}",
            self.recursion_height(),
            index
        );
        let callback = |_: &TreeIndex| new_value;
        self.access(index, callback, rng)
    }

    fn access<R: RngCore + CryptoRng, F: Fn(&TreeIndex) -> TreeIndex>(
        &mut self,
        index: Address,
        callback: F,
        rng: &mut R,
    ) -> TreeIndex {
        assert!(index < self.block_capacity());

        let address_of_block = AddressOram::<B, Z>::address_of_block(index);
        let address_within_block = AddressOram::<B, Z>::address_within_block(index);

        match self {
            // Base case: index into a linear-time ORAM.
            AddressOram::Base(linear_oram) => {
                assert_eq!(0, address_of_block);
                assert_eq!(index, address_within_block);
                linear_oram.access(index, callback, rng)
            }

            // Recursive case:
            // (1) split the address into an ORAM address (`address_of_block`) and an offset within the block (`address_within_block`)
            // (2) Recursively access the block at `address_of_block`, using a callback which updates only the address of interest in that block.
            // (3) Return the address of interest from the block.
            AddressOram::Recursive(block_oram) => {
                let block_callback = |block: &AddressOramBlock<B>| {
                    let mut result: AddressOramBlock<B> = *block;
                    for i in 0..block.data.len() {
                        let index_matches = i.ct_eq(&address_within_block);
                        let position_to_write = callback(&block.data[i]);
                        result.data[i].conditional_assign(&position_to_write, index_matches);
                    }
                    result
                };

                let block = block_oram.access(address_of_block, block_callback, rng);

                let mut result = u64::default();
                for i in 0..block.data.len() {
                    let index_matches = i.ct_eq(&address_within_block);
                    result.conditional_assign(&block.data[i], index_matches);
                }

                result
            }
        }
    }
}

#[cfg(test)]
mod tests {

    mod address_oram_tests {
        use core::iter::zip;

        use super::super::*;
        use crate::block_value::BlockValue;
        use crate::path_oram::*;
        use crate::test_utils::*;

        type ConcreteAddressOram<const AB: BlockSize, V> =
            VecPathOram<V, DEFAULT_BLOCKS_PER_BUCKET, AddressOram<AB, DEFAULT_BLOCKS_PER_BUCKET>>;

        create_correctness_tests_for_oram_type!(ConcreteAddressOram, AddressOramBlock);

        // Test that the stash size is not growing too large.
        type CAOStashSizeMonitor<const AB: BlockSize, V> =
            StashSizeMonitor<ConcreteAddressOram<AB, V>>;
        create_correctness_tests_for_oram_type!(CAOStashSizeMonitor, AddressOramBlock);

        // Test that the total number of non-dummy blocks in the ORAM stays constant.
        type CAOConstantOccupancyMonitor<const AB: BlockSize, V> =
            ConstantOccupancyMonitor<ConcreteAddressOram<AB, V>>;
        create_correctness_tests_for_oram_type!(CAOConstantOccupancyMonitor, AddressOramBlock);

        // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
        type CAOPhysicalAccessCountMonitor<const AB: BlockSize, V> =
            PhysicalAccessCountMonitor<ConcreteAddressOram<AB, V>>;
        create_correctness_tests_for_oram_type!(CAOPhysicalAccessCountMonitor, AddressOramBlock);

        // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
        #[derive(Debug)]
        struct CAOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
            oram: ConcreteAddressOram<B, V>,
        }
        create_statistics_test_for_oram_type!(CAOAccessDistributionTester, BlockValue);
    }

    mod block_oram_tests {
        mod address_oram_tests {
            use core::iter::zip;

            use crate::block_value::BlockValue;
            use crate::path_oram::*;
            use crate::test_utils::*;
            use crate::*;
            use recursive_insecure_path_oram::*;

            type ConcreteBlockOram<const B: BlockSize, V> =
                BlockOram<B, V, DEFAULT_BLOCKS_PER_BUCKET>;

            create_correctness_tests_for_oram_type!(ConcreteBlockOram, BlockValue);

            // Test that the stash size is not growing too large.
            type CBOStashSizeMonitor<const AB: BlockSize, V> =
                StashSizeMonitor<ConcreteBlockOram<AB, V>>;
            create_correctness_tests_for_oram_type!(CBOStashSizeMonitor, BlockValue);

            // Test that the total number of non-dummy blocks in the ORAM stays constant.
            type CBOConstantOccupancyMonitor<const AB: BlockSize, V> =
                ConstantOccupancyMonitor<ConcreteBlockOram<AB, V>>;
            create_correctness_tests_for_oram_type!(CBOConstantOccupancyMonitor, BlockValue);

            // Test that the number of physical accesses resulting from ORAM accesses is exactly as expected.
            type CBOCountPhysicalAccessesMonitor<const AB: BlockSize, V> =
                PhysicalAccessCountMonitor<ConcreteBlockOram<AB, V>>;
            create_correctness_tests_for_oram_type!(CBOCountPhysicalAccessesMonitor, BlockValue);

            // Test that the distribution of ORAM accesses across leaves is close to the expected (uniform) distribution.
            #[derive(Debug)]
            struct CBOAccessDistributionTester<const B: BlockSize, V: OramBlock> {
                oram: ConcreteBlockOram<B, V>,
            }
            create_statistics_test_for_oram_type!(CBOAccessDistributionTester, BlockValue);
        }
    }
}
