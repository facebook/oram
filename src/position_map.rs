// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A recursive Path ORAM position map data structure.

use super::path_oram::PathOram;
use crate::bucket::PositionBlock;
use crate::OramError;
use crate::{
    database::SimpleDatabase, linear_time_oram::LinearTimeOram, utils::TreeIndex, Address,
    BlockSize, BucketSize, Oram, OramBlock,
};
use log::debug;
use rand::{CryptoRng, RngCore};
use subtle::{ConditionallySelectable, ConstantTimeEq};

const RECURSION_THRESHOLD: u64 = 1 << 12;

impl<const AB: BlockSize, V: OramBlock, const Z: BucketSize> PathOram<V, Z, AB> {
    pub(crate) fn recursion_height(&self) -> usize {
        self.position_map.recursion_height()
    }
}

/// A recursive Path ORAM position map data structure. `AB` is the number of addresses stored in each ORAM block.
#[derive(Debug)]
pub enum PositionMap<const AB: BlockSize, const Z: BucketSize> {
    /// A simple, linear-time `AddressOram`.
    Base(LinearTimeOram<SimpleDatabase<PositionBlock<AB>>>),
    /// A recursive `AddressOram` whose position map is also an `AddressOram`.
    Recursive(Box<PathOram<PositionBlock<AB>, Z, AB>>),
}
impl<const AB: BlockSize, const Z: BucketSize> PositionMap<AB, Z> {
    pub(crate) fn recursion_height(&self) -> usize {
        match self {
            PositionMap::Base(_) => 0,
            PositionMap::Recursive(inner) => 1 + inner.recursion_height(),
        }
    }

    fn address_of_block(address: Address) -> Address {
        let block_address_bits = AB.ilog2();
        address >> block_address_bits
    }

    fn address_within_block(address: Address) -> Result<usize, OramError> {
        let block_address_bits = AB.ilog2();
        let shift: usize = (Address::BITS - block_address_bits).try_into()?;
        Ok(((address << shift) >> shift).try_into()?)
    }
}

impl<const AB: BlockSize, const Z: BucketSize> PositionMap<AB, Z> {
    pub fn write_position_block<R: RngCore + CryptoRng>(
        &mut self,
        address: Address,
        position_block: PositionBlock<AB>,
        rng: &mut R,
    ) -> Result<(), OramError> {
        let address_of_block = PositionMap::<AB, Z>::address_of_block(address);

        match self {
            PositionMap::Base(linear_oram) => {
                linear_oram.write(address_of_block, position_block, rng)?;
            }

            PositionMap::Recursive(block_oram) => {
                block_oram.write(address_of_block, position_block, rng)?;
            }
        }

        Ok(())
    }
}

impl<const AB: BlockSize, const Z: BucketSize> Oram<TreeIndex> for PositionMap<AB, Z> {
    fn new<R: CryptoRng + RngCore>(
        number_of_addresses: Address,
        rng: &mut R,
    ) -> Result<Self, OramError> {
        debug!(
            "Oram::new -- AddressOram(B = {}, Z = {}, C = {})",
            AB, Z, number_of_addresses
        );

        if (AB < 2) | (!AB.is_power_of_two()) {
            return Err(OramError::InvalidConfigurationError);
        }

        let ab_address: Address = AB.try_into()?;
        if number_of_addresses / ab_address <= RECURSION_THRESHOLD {
            let mut block_capacity = number_of_addresses / ab_address;
            if number_of_addresses % ab_address > 0 {
                block_capacity += 1;
            }
            Ok(Self::Base(LinearTimeOram::new(block_capacity, rng)?))
        } else {
            let block_capacity = number_of_addresses / ab_address;
            Ok(Self::Recursive(Box::new(PathOram::new(
                block_capacity,
                rng,
            )?)))
        }
    }

    fn block_capacity(&self) -> Result<Address, OramError> {
        match self {
            PositionMap::Base(linear_oram) => linear_oram.block_capacity(),
            PositionMap::Recursive(block_oram) => {
                let ab_address: Address = AB.try_into()?;
                Ok(block_oram.block_capacity()? * ab_address)
            }
        }
    }

    // Overwriting default method for logging purposes.
    fn read<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        rng: &mut R,
    ) -> Result<TreeIndex, OramError> {
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
    ) -> Result<TreeIndex, OramError> {
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
        address: Address,
        callback: F,
        rng: &mut R,
    ) -> Result<TreeIndex, OramError> {
        let address_of_block = PositionMap::<AB, Z>::address_of_block(address);
        let address_within_block = PositionMap::<AB, Z>::address_within_block(address)?;

        let block_callback = |block: &PositionBlock<AB>| {
            let mut result: PositionBlock<AB> = *block;
            for i in 0..block.data.len() {
                let index_matches = i.ct_eq(&address_within_block);
                let position_to_write = callback(&block.data[i]);
                result.data[i].conditional_assign(&position_to_write, index_matches);
            }
            result
        };

        match self {
            // Base case: index into a linear-time ORAM.
            PositionMap::Base(linear_oram) => {
                let block = linear_oram.access(address_of_block, block_callback, rng)?;
                Ok(block.data[address_within_block])
            }

            // Recursive case:
            // (1) split the address into an ORAM address (`address_of_block`) and an offset within the block (`address_within_block`)
            // (2) Recursively access the block at `address_of_block`, using a callback which updates only the address of interest in that block.
            // (3) Return the address of interest from the block.
            PositionMap::Recursive(block_oram) => {
                let block = block_oram.access(address_of_block, block_callback, rng)?;

                let mut result = u64::default();
                for i in 0..block.data.len() {
                    let index_matches = i.ct_eq(&address_within_block);
                    result.conditional_assign(&block.data[i], index_matches);
                }

                Ok(result)
            }
        }
    }
}
