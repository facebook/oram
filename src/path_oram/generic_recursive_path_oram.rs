// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An insecure implementation of Path ORAM with "client-side" stash and recursive position map.

use super::generic_path_oram::GenericPathOram;
use super::{address_oram_block::AddressOramBlock, stash::Stash, TreeIndex};
use crate::{
    database::SimpleDatabase, linear_time_oram::LinearTimeOram, Address, BlockSize, BucketSize,
    Oram, OramBlock,
};
use log::debug;
use rand::{CryptoRng, RngCore};
use subtle::{ConditionallySelectable, ConstantTimeEq};

/// A Path ORAM with a recursive position map and "client-side" stash.
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
pub type BlockOram<const AB: BlockSize, V, const Z: BucketSize, S1, S2> =
    GenericPathOram<V, Z, AddressOram<AB, Z, S1>, S2>;

impl<
        const AB: BlockSize,
        V: OramBlock,
        const Z: BucketSize,
        S1: Stash<AddressOramBlock<AB>> + std::fmt::Debug,
        S2: Stash<V>,
    > BlockOram<AB, V, Z, S1, S2>
{
    pub(crate) fn recursion_height(&self) -> usize {
        self.position_map.recursion_height()
    }
}

/// An `Oram` intended for use as a position map. `AB` is the block size in addresses.
#[derive(Debug)]
pub enum AddressOram<
    const AB: BlockSize,
    const Z: BucketSize,
    S: Stash<AddressOramBlock<AB>> + std::fmt::Debug,
> {
    /// A simple, linear-time `AddressOram`.
    Base(LinearTimeOram<SimpleDatabase<TreeIndex>>),
    /// A recursive `AddressOram` whose position map is a `VecPathOram`.
    Recursive(Box<GenericPathOram<AddressOramBlock<AB>, Z, AddressOram<AB, Z, S>, S>>),
}

impl<
        const AB: BlockSize,
        const Z: BucketSize,
        S: Stash<AddressOramBlock<AB>> + std::fmt::Debug,
    > AddressOram<AB, Z, S>
{
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

impl<const B: BlockSize, const Z: BucketSize, S: Stash<AddressOramBlock<B>> + std::fmt::Debug>
    Oram<TreeIndex> for AddressOram<B, Z, S>
{
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
            Self::Recursive(Box::new(GenericPathOram::new(block_capacity, rng)))
        }
    }

    fn block_capacity(&self) -> Address {
        match self {
            AddressOram::Base(linear_oram) => linear_oram.block_capacity(),
            AddressOram::Recursive(block_oram) => block_oram.block_capacity() * B,
        }
    }

    // Overwriting default method for logging purposes.
    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> TreeIndex {
        log::debug!(
            "Level {} AddressORAM read: {}",
            self.recursion_height(),
            index
        );
        let callback = |x: &TreeIndex| *x;
        self.access(index, callback, rng)
    }

    // Overwriting default method for logging purposes.
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

        let address_of_block = AddressOram::<B, Z, S>::address_of_block(index);
        let address_within_block = AddressOram::<B, Z, S>::address_within_block(index);

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
