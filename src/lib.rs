// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! An implementation of Oblivious RAM

#![warn(clippy::cargo, clippy::doc_markdown, missing_docs, rustdoc::all)]

use aligned::{Aligned, A64};
use rand::{
    distributions::{Distribution, Standard},
    CryptoRng, Rng, RngCore,
};
use std::ops::BitAnd;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeLess, CtOption};

/// The numeric type used to specify the size of an ORAM in blocks, and to index into the ORAM.
pub type IndexType = usize;
/// The numeric type used to specify the size of each block of the ORAM in bytes.
pub type BlockSizeType = usize;

/// Represents an oblivious RAM (ORAM) mapping `IndexType` addresses to `BlockValue` values.
/// `B` represents the size of each block of the ORAM in bytes.
pub trait Oram<const B: BlockSizeType> {
    /// Returns a new ORAM mapping addresses `0 <= address <= block_capacity` to default `BlockValue` values.
    fn new<R: RngCore + CryptoRng>(block_capacity: IndexType, rng: &mut R) -> Self;
    /// Returns the capacity in blocks of this ORAM.
    fn block_capacity(&self) -> IndexType;

    /// Returns the size in bytes of each block of this ORAM.
    fn block_size(&self) -> BlockSizeType;

    /// Performs a (oblivious) ORAM access.
    /// If `optional_new_value.is_some()`, writes  `optional_new_value.unwrap()` into `index`.
    /// Returns the value previously stored at `index`.
    fn access<R: RngCore + CryptoRng>(
        &mut self,
        index: IndexType,
        optional_new_value: CtOption<BlockValue<B>>,
        rng: &mut R,
    ) -> BlockValue<B>;

    /// Obliviously reads the value stored at `index`.
    fn read<R: RngCore + CryptoRng>(&mut self, index: IndexType, rng: &mut R) -> BlockValue<B> {
        let ct_none = CtOption::new(BlockValue::default(), 0.into());
        self.access(index, ct_none, rng)
    }

    /// Obliviously writes the value stored at `index`.
    fn write<R: RngCore + CryptoRng>(
        &mut self,
        index: IndexType,
        new_value: BlockValue<B>,
        rng: &mut R,
    ) {
        let ct_some_new_value = CtOption::new(new_value, 1.into());
        self.access(index, ct_some_new_value, rng);
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
    /// Reads the value stored at `index`, without any instrumentation or other side effects.
    fn write(&mut self, index: IndexType, value: V);
}

/// A simple Database that stores its data as a Vec.
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

/// A simple ORAM that, for each access, ensures obliviousness by making a complete pass over the database,
/// reading and writing each memory location.
pub struct LinearTimeOram<DB> {
    /// The memory of the ORAM.
    // Made this public for benchmarking, which ideally, I would not need to do.
    pub physical_memory: DB,
}

impl<const B: BlockSizeType, DB: Database<BlockValue<B>>> Oram<B> for LinearTimeOram<DB> {
    fn new<R: RngCore + CryptoRng>(block_capacity: IndexType, _: &mut R) -> Self {
        Self {
            physical_memory: DB::new(block_capacity),
        }
    }

    fn block_size(&self) -> BlockSizeType {
        B
    }

    fn access<R: RngCore + CryptoRng>(
        &mut self,
        index: IndexType,
        optional_new_value: CtOption<BlockValue<B>>,
        _: &mut R,
    ) -> BlockValue<B> {
        // Note: index and optional_new_value should be considered secret for the purposes of constant-time operations.

        // TODO(#6): Handle malformed input in a more robust way.
        let index_in_bounds: bool = (index as u128)
            .ct_lt(&(self.block_capacity() as u128))
            .into();

        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not. See also Issue #6.
        assert!(index_in_bounds);

        // This is a dummy value which will always be overwritten.
        let mut result = BlockValue::default();

        for i in 0..self.physical_memory.capacity() {
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
        self.physical_memory.capacity()
    }
}

/// A type alias for a simple `LinearTimeOram` monomorphization.
pub type LinearOram<const B: usize> = LinearTimeOram<CountAccessesDatabase<BlockValue<B>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use std::mem;

    #[test]
    fn check_alignment() {
        let irrelevant_capacity = 64;
        let expected_alignment = 64;
        let database = SimpleDatabase::<BlockValue<64>>::new(irrelevant_capacity);
        for block in &database.0 {
            assert_eq!(mem::align_of_val(block), expected_alignment);
        }
    }

    fn test_correctness_random_workload<const B: usize>(capacity: usize, num_operations: u32) {
        let mut rng = StdRng::seed_from_u64(0);

        let mut oram: LinearOram<B> = LinearOram::new(capacity, &mut rng);
        let mut mirror_array = vec![BlockValue::default(); capacity];

        for _ in 0..num_operations {
            let random_index = rng.gen_range(0..capacity);
            let random_block_value = rng.gen();

            let read_versus_write: bool = rng.gen();

            if read_versus_write {
                assert_eq!(
                    oram.read(random_index, &mut rng),
                    mirror_array[random_index]
                );
            } else {
                oram.write(random_index, random_block_value, &mut rng);
                mirror_array[random_index] = random_block_value;
            }
        }

        for index in 0..capacity {
            assert_eq!(oram.read(index, &mut rng), mirror_array[index], "{index}")
        }
    }

    #[test]
    fn test_correctness_random_workload_1_64_10000() {
        test_correctness_random_workload::<1>(64, 10000);
    }

    #[test]
    fn test_correctness_random_workload_64_1_10000() {
        test_correctness_random_workload::<64>(1, 10000);
    }

    #[test]
    fn test_correctness_random_workload_64_64_10000() {
        test_correctness_random_workload::<64>(64, 10000);
    }

    #[test]
    fn test_correctness_random_workload_64_256_10000() {
        test_correctness_random_workload::<64>(256, 10000);
    }

    #[test]
    fn test_correctness_random_workload_4096_64_1000() {
        test_correctness_random_workload::<4096>(200, 1000);
    }

    #[test]
    fn test_correctness_random_workload_4096_256_1000() {
        test_correctness_random_workload::<4096>(256, 1000);
    }

    fn test_correctness_linear_workload<const B: usize>(capacity: usize, num_passes: u32) {
        let mut rng = StdRng::seed_from_u64(0);

        let mut oram: LinearOram<B> = LinearOram::new(capacity, &mut rng);

        let mut mirror_array = vec![BlockValue::default(); capacity];

        for _ in 0..num_passes {
            for index in 0..capacity {
                let random_block_value = rng.gen();

                let read_versus_write: bool = rng.gen();

                if read_versus_write {
                    assert_eq!(oram.read(index, &mut rng), mirror_array[index]);
                } else {
                    oram.write(index, random_block_value, &mut rng);
                    mirror_array[index] = random_block_value;
                }
            }
        }

        for index in 0..capacity {
            assert_eq!(oram.read(index, &mut rng), mirror_array[index], "{index}")
        }
    }

    #[test]
    fn test_correctness_linear_workload_1_64_100() {
        test_correctness_linear_workload::<1>(64, 100);
    }

    #[test]
    fn test_correctness_linear_workload_64_1_100() {
        test_correctness_linear_workload::<64>(1, 100);
    }

    #[test]
    fn test_correctness_linear_workload_64_64_100() {
        test_correctness_linear_workload::<64>(64, 100);
    }

    #[test]
    fn test_correctness_linear_workload_64_256_100() {
        test_correctness_linear_workload::<64>(256, 100);
    }

    #[test]
    fn test_correctness_linear_workload_4096_64_10() {
        test_correctness_linear_workload::<4096>(64, 10);
    }

    #[test]
    fn test_correctness_linear_workload_4096_256_2() {
        test_correctness_linear_workload::<4096>(256, 2);
    }
}
