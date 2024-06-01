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
use subtle::{Choice, ConditionallySelectable, CtOption};

type BlockSizeType = usize;
type IndexType = usize;

// For debugging, we can keep this value small. For production, probably 4096 (a 4KB memory page)
// makes the most sense.
const BLOCK_SIZE: BlockSizeType = 64;

/// Represents an oblivious RAM mapping `IndexType` addresses to `BlockValue` values.
pub trait ORAM {
    /// Returns a new ORAM mapping addresses `0 <= address <= block_capacity` to default `BlockValue` values.
    fn new(block_capacity: IndexType) -> Self;

    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> IndexType;

    /// Performs an ORAM access.
    /// If `optional_new_value.is_some()`, writes  `optional_new_value.unwrap()` into `index`.
    /// Returns the value previously stored at `index`.
    fn access(&mut self, index: IndexType, optional_new_value: CtOption<BlockValue>) -> BlockValue;

    /// Obliviously reads the value stored at `index`.
    fn read(&mut self, index: IndexType) -> BlockValue {
        let ct_none = CtOption::new(BlockValue::default(), 0.into());
        self.access(index, ct_none)
    }

    /// Obliviously writes the value stored at `index`.
    fn write(&mut self, index: IndexType, new_value: BlockValue) {
        let ct_some_new_value = CtOption::new(new_value, 1.into());
        self.access(index, ct_some_new_value);
    }
}

/// The basic unit of memory accessible by ORAM operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockValue([u8; BLOCK_SIZE]);

impl Default for BlockValue {
    fn default() -> Self {
        BlockValue([0; BLOCK_SIZE])
    }
}

impl ConditionallySelectable for BlockValue {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = BlockValue::default();
        for i in 0..BLOCK_SIZE {
            result.0[i] = u8::conditional_select(&a.0[i], &b.0[i], choice);
        }
        result
    }
}

impl Distribution<BlockValue> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockValue {
        let mut result = BlockValue::default();
        for i in 0..BLOCK_SIZE {
            result.0[i] = rng.gen();
        }
        result
    }
}

// Spencer: Added a simple Memory trait to model the memory controller the TEE is interacting with.
trait Memory<V: Default + Copy> {
    fn new(number_of_addresses: IndexType) -> Self;
    fn len(&self) -> IndexType;
    fn read(&self, index: IndexType) -> V;
    fn write(&mut self, index: IndexType, value: V);
}

struct SimpleMemory<V>(Vec<V>);

impl<V: Default + Copy> Memory<V> for SimpleMemory<V> {
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

struct LinearTimeORAM {
    memory: SimpleMemory<BlockValue>,
}

impl ORAM for LinearTimeORAM {
    fn new(block_capacity: IndexType) -> Self {
        Self {
            memory: SimpleMemory::new(block_capacity),
        }
    }

    fn access(&mut self, index: IndexType, optional_new_value: CtOption<BlockValue>) -> BlockValue {
        assert!(index < self.block_capacity());

        // This is a dummy value which will always be overwritten.
        let mut result = BlockValue::default();

        for i in 0..self.memory.len() {
            // Read from memory
            let entry = self.memory.read(i);

            // Client-side processing
            let is_requested_index: Choice = (u8::from(index == i)).into();

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
            self.memory.write(i, potentially_updated_value);
        }
        result
    }

    fn block_capacity(&self) -> IndexType {
        self.memory.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn simple_read_write() {
        let mut oram = LinearTimeORAM::new(16);
        let written_value = BlockValue([1; BLOCK_SIZE]);
        oram.write(0, written_value);
        let read_value = oram.read(0);
        assert_eq!(written_value, read_value);
    }
}
