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
    Rng,
};
use subtle::{Choice, ConditionallySelectable, CtOption};

pub mod linear_time_oram;
pub mod simple_insecure_path_oram;
mod test_utils;

/// The numeric type used to specify the size of an ORAM in blocks, and to index into the ORAM.
pub type IndexType = usize;
/// The numeric type used to specify the size of each block of the ORAM in bytes.
pub type BlockSizeType = usize;


type TreeIndex = u64;

type TreeHeight = u32;
const MAXIMUM_TREE_HEIGHT: TreeHeight = 63;

/// The numeric type used to specify the size of an ORAM bucket in blocks.
pub type BucketSizeType = usize;
/// The parameter "Z" from the Path ORAM literature that sets the number of blocks per bucket; typical values are 3 or 4.
/// Here we adopt the more conservative setting of 4.
pub const DEFAULT_BLOCKS_PER_BUCKET: BucketSizeType = 4;

/// Represents an oblivious RAM (ORAM) mapping `IndexType` addresses to `BlockValue` values.
/// `B` represents the size of each block of the ORAM in bytes.
pub trait Oram<const B: BlockSizeType, R: Rng> {
    /// Returns a new `Oram` mapping addresses `0 <= address <= block_capacity` to default `BlockValue` values.
    fn new(block_capacity: IndexType, rng: R) -> Self
    where
        Self: Sized;

    /// Returns the capacity in blocks of this `Oram`.
    fn block_capacity(&self) -> IndexType;

    /// Returns the size in bytes of each block of this `Oram`.
    fn block_size(&self) -> BlockSizeType;

    /// Performs a (oblivious) ORAM access.
    /// If `optional_new_value.is_some()`, writes  `optional_new_value.unwrap()` into `index`.
    /// Returns the value previously stored at `index`.
    fn access(
        &mut self,
        index: IndexType,
        optional_new_value: CtOption<BlockValue<B>>,
    ) -> BlockValue<B>;

    /// Obliviously reads the value stored at `index`.
    fn read(&mut self, index: IndexType) -> BlockValue<B> {
        let ct_none = CtOption::new(BlockValue::default(), 0.into());
        self.access(index, ct_none)
    }

    /// Obliviously writes the value stored at `index`.
    fn write(&mut self, index: IndexType, new_value: BlockValue<B>) {
        let ct_some_new_value = CtOption::new(new_value, 1.into());
        self.access(index, ct_some_new_value);
    }
}

/// The smallest unit of memory readable/writable by the ORAM.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockValue<const B: BlockSizeType>(Aligned<A64, [u8; B]>);

impl<const B: BlockSizeType> BlockValue<B> {
    /// Returns the length in bytes of this `BlockValue`.
    pub fn byte_length(&self) -> BlockSizeType {
        B
    }

    /// Instantiates a `BlockValue` from an array of `BLOCK_SIZE` bytes.
    pub fn from_byte_array(data: [u8; B]) -> Self {
        Self(Aligned(data))
    }
}

impl<const B: BlockSizeType> From<BlockValue<B>> for [u8; B] {
    fn from(value: BlockValue<B>) -> Self {
        *value.0
    }
}

impl<const B: BlockSizeType> Default for BlockValue<B> {
    fn default() -> Self {
        BlockValue::<B>(Aligned([0u8; B]))
    }
}

impl<const B: BlockSizeType> ConditionallySelectable for BlockValue<B> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = BlockValue::default();
        for i in 0..a.byte_length() {
            result.0[i] = u8::conditional_select(&a.0[i], &b.0[i], choice);
        }
        result
    }
}

impl<const B: BlockSizeType> Distribution<BlockValue<B>> for Standard {
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
    fn new(number_of_addresses: IndexType) -> Self;
    /// Returns the number of values stored by `self`.
    fn capacity(&self) -> IndexType;
    /// Reads the value stored at `index`.
    fn read(&mut self, index: IndexType) -> V;
    /// Writes the value stored at `index`.
    fn write(&mut self, index: IndexType, value: V);
}

/// A simple Database that stores its data as a Vec.
#[derive(Debug)]
pub struct SimpleDatabase<V>(Vec<V>);

impl<V: Default + Copy> Database<V> for SimpleDatabase<V> {
    fn new(number_of_addresses: IndexType) -> Self {
        Self(vec![V::default(); number_of_addresses])
    }

    fn capacity(&self) -> IndexType {
        self.0.len()
    }

    fn read(&mut self, index: IndexType) -> V {
        self.0[index]
    }

    fn write(&mut self, index: IndexType, value: V) {
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
    fn new(number_of_addresses: IndexType) -> Self {
        Self {
            data: SimpleDatabase::new(number_of_addresses),
            read_count: 0,
            write_count: 0,
        }
    }

    fn read(&mut self, index: IndexType) -> V {
        self.read_count += 1;
        self.data.read(index)
    }

    fn write(&mut self, index: IndexType, value: V) {
        self.write_count += 1;
        self.data.write(index, value);
    }

    fn capacity(&self) -> IndexType {
        self.data.capacity()
    }
}
