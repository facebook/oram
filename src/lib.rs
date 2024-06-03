// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Oblivious RAM

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::ops::BitAnd;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeLess, CtOption};
use generic_array::{sequence::GenericSequence, ArrayLength, GenericArray, typenum::U64};

type BlockSizeType = usize;
type IndexType = usize;

// For debugging, we can keep this value small. For production, probably 4096 (a 4KB memory page)
// makes the most sense.
const BLOCK_SIZE: BlockSizeType = 64;

/// Represents an oblivious RAM mapping `IndexType` addresses to `BlockValue` values.
pub trait ORAM<B: ArrayLength> {
    /// Returns a new ORAM mapping addresses `0 <= address <= block_capacity` to default `BlockValue` values.
    fn new(block_capacity: IndexType) -> Self;

    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> IndexType;

    /// Performs a (oblivious) ORAM access.
    /// If `optional_new_value.is_some()`, writes  `optional_new_value.unwrap()` into `index`.
    /// Returns the value previously stored at `index`.
    fn access(&mut self, index: IndexType, optional_new_value: CtOption<BlockValue<B>>) -> BlockValue<B>;

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
#[derive(Clone, Debug, PartialEq)]
pub struct BlockValue<B: ArrayLength>(GenericArray<u8, B>);
impl<B: ArrayLength> Copy for BlockValue<B> where B::ArrayType<u8>: Copy {}

impl<B: ArrayLength> Default for BlockValue<B> {
    fn default() -> Self {
        BlockValue::<B>(GenericArray::generate(|_| 0))
    }
}

impl<B: ArrayLength> ConditionallySelectable for BlockValue<B> 
where <B as ArrayLength>::ArrayType<u8>: Copy
{
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = BlockValue::default();
        for i in 0..BLOCK_SIZE {
            result.0[i] = u8::conditional_select(&a.0[i], &b.0[i], choice);
        }
        result
    }
}

impl<B: ArrayLength> Distribution<BlockValue<B>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockValue<B> {
        let mut result = BlockValue::default();
        for i in 0..BLOCK_SIZE {
            result.0[i] = rng.gen();
        }
        result
    }
}

// A simple Memory trait to model the memory controller the TEE is interacting with.
trait Database<V: Default + Copy> {
    fn new(number_of_addresses: IndexType) -> Self;
    fn len(&self) -> IndexType;
    fn read(&self, index: IndexType) -> V;
    fn write(&mut self, index: IndexType, value: V);
}

struct SimpleDatabase<V>(Vec<V>);

impl<V: Default + Copy> Database<V> for SimpleDatabase<V> {
    fn new(number_of_addresses: IndexType) -> Self {
        Self(vec![V::default(); number_of_addresses])
    }

    fn len(&self) -> IndexType {
        self.0.len()
    }

    fn read(&self, index: IndexType) -> V {
        self.0[index]
    }

    fn write(&mut self, index: IndexType, value: V) {
        self.0[index] = value;
    }
}

struct LinearTimeORAM<B: ArrayLength> {
    physical_memory: SimpleDatabase<BlockValue<B>>,
}

impl <B: ArrayLength> ORAM<B> for LinearTimeORAM<B> 
where <B as ArrayLength>::ArrayType<u8>: Copy
{
    fn new(block_capacity: IndexType) -> Self {
        Self {
            physical_memory: SimpleDatabase::new(block_capacity),
        }
    }

    fn access(&mut self, index: IndexType, optional_new_value: CtOption<BlockValue<B>>) -> BlockValue<B> {
        // Note: index and optional_new_value should be considered secret for the purposes of constant-time operations.

        // TODO(#6): Handle malformed input in a more robust way.
        let index_in_bounds: bool = (index as u128)
            .ct_lt(&(self.block_capacity() as u128))
            .into();

        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not. See also Issue #6.
        assert!(index_in_bounds);

        // This is a dummy value which will always be overwritten.
        let mut result = BlockValue::default();

        for i in 0..self.physical_memory.len() {
            // Read from memory
            let entry = self.physical_memory.read(i);

            // Client-side processing
            // let is_requested_index: Choice = (u8::from(index == i)).into();
            let is_requested_index = (i as IndexType).ct_eq(&index);

            // Based on whether the loop counter matches the requested index,
            // conditionally read the value in memory into the result of the access.
            result.conditional_assign(&entry, is_requested_index);

            let oram_operation_is_write = optional_new_value.is_some();
            let should_write = is_requested_index.bitand(oram_operation_is_write);
            // Note that the unwrap_or_else method of CtOption is constant-time.
            let value_to_write = optional_new_value.unwrap_or_else(BlockValue::default);

            // Based on whether (1) the loop counter matches the requested index,
            // AND (2) this ORAM access is a write,
            // select the value to be written back out to memory to be either the original value
            // or the provided new value.
            let potentially_updated_value =
                BlockValue::conditional_select(&entry, &value_to_write, should_write);
            // End client-side processing

            // Write the (potentially) updated value back to memory.
            self.physical_memory.write(i, potentially_updated_value);
        }
        result
    }

    fn block_capacity(&self) -> IndexType {
        self.physical_memory.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn simple_read_write() {
        let mut oram: LinearTimeORAM<U64>= LinearTimeORAM::new(16);
        let written_value = BlockValue(GenericArray::generate(|_| 1));
        oram.write(0, written_value);
        let read_value = oram.read(0);
        assert_eq!(written_value, read_value);
    }

    #[test]
    fn check_correctness() {
        const BLOCK_CAPACITY: usize = 256;
        let num_operations = 10000;

        let mut rng = StdRng::seed_from_u64(0);

        let mut oram: LinearTimeORAM<U64> = LinearTimeORAM::new(BLOCK_CAPACITY);
        let mut mirror_array = [BlockValue::default(); BLOCK_CAPACITY];

        for _ in 0..num_operations {
            let random_index = rng.gen_range(0..BLOCK_CAPACITY);
            let random_block_value = rng.gen();

            let read_versus_write: bool = rng.gen();

            if read_versus_write {
                assert_eq!(oram.read(random_index), mirror_array[random_index]);
            } else {
                oram.write(random_index, random_block_value);
                mirror_array[random_index] = random_block_value;
            }
        }

        for index in 0..BLOCK_CAPACITY {
            assert_eq!(oram.read(index), mirror_array[index], "{index}")
        }
    }
}
