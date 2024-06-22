// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Oblivious RAM.

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use aligned::{Aligned, A64};
use rand::{
    distributions::{Distribution, Standard},
    CryptoRng, Rng, RngCore,
};
use subtle::{Choice, ConditionallySelectable, CtOption};

pub mod linear_time_oram;
pub mod simple_insecure_path_oram;

#[cfg(test)]
mod test_utils;

/// The numeric type used to specify the size of an ORAM in blocks, and to index into the ORAM.
pub type Address = usize;
/// The numeric type used to specify the size of each block of the ORAM in bytes.
pub type BlockSize = usize;

type TreeIndex = u64;

type TreeHeight = u32;
const MAXIMUM_TREE_HEIGHT: TreeHeight = 63;

/// The numeric type used to specify the size of an ORAM bucket in blocks.
pub type BucketSizeType = usize;
/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSizeType = 4;

/// TODO
pub trait OramBlock:
    Copy + Clone + std::fmt::Debug + Default + PartialEq + ConditionallySelectable
{
}

/// Represents an oblivious RAM (ORAM) mapping `IndexType` addresses to `BlockValue` values.
/// `B` represents the size of each block of the ORAM in bytes.
pub trait Oram<V: OramBlock> {
    /// Returns a new ORAM mapping addresses `0 <= address <= block_capacity` to default `BlockValue` values.
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, rng: &mut R) -> Self;
    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> Address;

    /// Returns the size in bytes of each block of this `Oram`.
    // fn block_size(&self) -> BlockSize;

    /// Performs a (oblivious) ORAM access.
    /// If `optional_new_value.is_some()`, writes  `optional_new_value.unwrap()` into `index`.
    /// Returns the value previously stored at `index`.
    fn access<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        optional_new_value: CtOption<V>,
        rng: &mut R,
    ) -> V;

    /// Obliviously reads the value stored at `index`.
    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> V {
        let ct_none = CtOption::new(V::default(), 0.into());
        self.access(index, ct_none, rng)
    }

    /// Obliviously writes the value stored at `index`.
    fn write<R: RngCore + CryptoRng>(&mut self, index: Address, new_value: V, rng: &mut R) {
        let ct_some_new_value = CtOption::new(new_value, 1.into());
        self.access(index, ct_some_new_value, rng);
    }
}

/// The smallest unit of memory readable/writable by the ORAM.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockValue<const B: BlockSize>(Aligned<A64, [u8; B]>);

impl<const B: BlockSize> OramBlock for BlockValue<B> {}

impl<const B: BlockSize> BlockValue<B> {
    /// Returns the length in bytes of this `BlockValue`.
    pub fn byte_length(&self) -> BlockSize {
        B
    }

    /// Instantiates a `BlockValue` from an array of `BLOCK_SIZE` bytes.
    pub fn from_byte_array(data: [u8; B]) -> Self {
        Self(Aligned(data))
    }
}

impl<const B: BlockSize> From<BlockValue<B>> for [u8; B] {
    fn from(value: BlockValue<B>) -> Self {
        *value.0
    }
}

impl<const B: BlockSize> Default for BlockValue<B> {
    fn default() -> Self {
        BlockValue::<B>(Aligned([0u8; B]))
    }
}

impl<const B: BlockSize> ConditionallySelectable for BlockValue<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = BlockValue::default();
        for i in 0..a.byte_length() {
            result.0[i] = u8::conditional_select(&a.0[i], &b.0[i], choice);
        }
        result
    }
}

impl<const B: BlockSize> Distribution<BlockValue<B>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockValue<B> {
        let mut result = BlockValue::default();
        for i in 0..result.byte_length() {
            result.0[i] = rng.gen();
        }
        result
    }
}

/// A simple Memory trait to model the memory controller the TEE is interacting with.
pub trait Database<V: Default + Copy>: Sized {
    /// Returns a new `Database` filled with default values.
    fn new(number_of_addresses: Address) -> Self;
    /// Returns the number of values stored by `self`.
    fn capacity(&self) -> Address;
    /// Reads the value stored at `index`.
    fn read(&mut self, index: Address) -> V;
    /// Writes the value stored at `index`.
    fn write(&mut self, index: Address, value: V);
}

/// A simple Database that stores its data as a Vec.
#[derive(Debug)]
pub struct SimpleDatabase<V>(Vec<V>);

impl<V: Default + Copy> Database<V> for SimpleDatabase<V> {
    fn new(number_of_addresses: Address) -> Self {
        Self(vec![V::default(); number_of_addresses])
    }

    fn capacity(&self) -> Address {
        self.0.len()
    }

    fn read(&mut self, index: Address) -> V {
        self.0[index]
    }

    fn write(&mut self, index: Address, value: V) {
        self.0[index] = value;
    }
}

/// A Database that counts reads and writes.
#[derive(Debug)]
pub struct CountAccessesDatabase<V> {
    data: SimpleDatabase<V>,
    read_count: u128,
    write_count: u128,
}

impl<V> CountAccessesDatabase<V> {
    /// Returns the total number of reads to the database.
    pub fn get_read_count(&self) -> u128 {
        self.read_count
    }

    /// Returns the total number of writes to the database.
    pub fn get_write_count(&self) -> u128 {
        self.write_count
    }
}

impl<V: Default + Copy> Database<V> for CountAccessesDatabase<V> {
    fn new(number_of_addresses: Address) -> Self {
        Self {
            data: SimpleDatabase::new(number_of_addresses),
            read_count: 0,
            write_count: 0,
        }
    }

    fn read(&mut self, index: Address) -> V {
        self.read_count += 1;
        self.data.read(index)
    }

    fn write(&mut self, index: Address, value: V) {
        self.write_count += 1;
        self.data.write(index, value);
    }

    fn capacity(&self) -> Address {
        self.data.capacity()
    }
}
