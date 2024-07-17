// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Memory abstractions for Oblivious RAM.

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use crate::{Address, Oram, OramBlock, ProtocolError};
use duplicate::duplicate_item;
use rand::{CryptoRng, RngCore};

/// A simple Memory trait to model the memory controller the TEE is interacting with.
pub trait Database<V: OramBlock>
where
    Self: Sized,
{
    /// Returns a new `Database` filled with default values.
    fn new(number_of_addresses: Address) -> Result<Self, ProtocolError>;
    /// Returns the number of values stored by `self`.
    fn capacity(&self) -> Result<Address, ProtocolError>;
    /// Reads the value stored at `index`.
    fn read_db(&mut self, index: Address) -> Result<V, ProtocolError>;
    /// Writes the value stored at `index`.
    fn write_db(&mut self, index: Address, value: V) -> Result<V, ProtocolError>;
}

/// A simple Database that stores its data as a Vec.
#[derive(Debug)]
pub struct SimpleDatabase<V>(Vec<V>);

impl<V: OramBlock> Database<V> for SimpleDatabase<V> {
    fn new(number_of_addresses: Address) -> Result<Self, ProtocolError> {
        Ok(Self(vec![V::default(); number_of_addresses.try_into()?]))
    }

    fn capacity(&self) -> Result<Address, ProtocolError> {
        Ok(self.0.len().try_into()?)
    }

    fn read_db(&mut self, index: Address) -> Result<V, ProtocolError> {
        let index: usize = index.try_into()?;
        Ok(self.0[index])
    }

    fn write_db(&mut self, index: Address, value: V) -> Result<V, ProtocolError> {
        let index: usize = index.try_into()?;
        let result = self.0[index];
        self.0[index] = value;
        Ok(result)
    }
}

/// A Database that counts reads and writes.
#[derive(Debug)]
pub struct CountAccessesDatabase<V> {
    data: SimpleDatabase<V>,
    /// `reads[i]` tracks the total number of reads made to index `i`.
    pub reads: Vec<u64>,
    /// `writes[i]` tracks the total number of writes made to index `i`.
    pub writes: Vec<u64>,
    // read_count: u128,
    // write_count: u128,
}

impl<V> CountAccessesDatabase<V> {
    /// Returns the total number of reads to the database.
    pub fn get_read_count(&self) -> u64 {
        self.reads.iter().sum()
    }

    /// Returns the total number of writes to the database.
    pub fn get_write_count(&self) -> u64 {
        self.writes.iter().sum()
    }
}

impl<V: OramBlock> Database<V> for CountAccessesDatabase<V> {
    fn new(number_of_addresses: Address) -> Result<Self, ProtocolError> {
        Ok(Self {
            data: Database::new(number_of_addresses)?,
            reads: vec![0u64; number_of_addresses.try_into()?],
            writes: vec![0u64; number_of_addresses.try_into()?],
        })
    }

    fn read_db(&mut self, index: Address) -> Result<V, ProtocolError> {
        log::debug!("Physical read -- {}", index);

        self.reads[usize::try_from(index)?] += 1;
        self.data.read_db(index)
    }

    fn write_db(&mut self, index: Address, value: V) -> Result<V, ProtocolError> {
        log::debug!("Physical write -- {}", index);

        self.writes[usize::try_from(index)?] += 1;
        self.data.write_db(index, value)
    }

    fn capacity(&self) -> Result<Address, ProtocolError> {
        self.data.capacity()
    }
}

// Implements `Oram`` for each `T: Database` so that the same correctness tests can be used for both.
#[duplicate_item(
    database_type;
    [SimpleDatabase];
    [CountAccessesDatabase];
)]
impl<V: OramBlock> Oram<V> for database_type<V> {
    fn new<R: RngCore + CryptoRng>(
        block_capacity: Address,
        _: &mut R,
    ) -> Result<Self, ProtocolError> {
        Database::new(block_capacity)
    }

    fn read<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        _: &mut R,
    ) -> Result<V, ProtocolError> {
        self.read_db(index)
    }

    fn write<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        new_value: V,
        _: &mut R,
    ) -> Result<V, ProtocolError> {
        self.write_db(index, new_value)
    }

    fn block_capacity(&self) -> Result<Address, ProtocolError> {
        Database::capacity(self)
    }

    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        _: &mut R,
    ) -> Result<V, ProtocolError> {
        let value = self.read_db(index)?;
        self.write_db(index, callback(&value))?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{CountAccessesDatabase, Database, SimpleDatabase};
    use crate::{block_value::BlockValue, test_utils::*, BlockSize};
    use std::mem;

    #[test]
    fn check_alignment() {
        init_logger();

        let irrelevant_capacity = 64;
        let expected_alignment = 64;
        let database =
            <SimpleDatabase<BlockValue<64>> as Database<BlockValue<64>>>::new(irrelevant_capacity)
                .unwrap();
        for block in &database.0 {
            assert_eq!(mem::align_of_val(block), expected_alignment);
        }
    }

    type TestingSimpleDatabase<const B: BlockSize, V> = SimpleDatabase<V>;
    type TestingCountAccessesDatabase<const B: BlockSize, V> = CountAccessesDatabase<V>;

    create_correctness_tests_for_oram_type!(TestingSimpleDatabase, BlockValue);
    create_correctness_tests_for_oram_type!(TestingCountAccessesDatabase, BlockValue);
}
