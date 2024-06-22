// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple linear-time implementation of Oblivious RAM.

use rand::{distributions::{Distribution, Standard}, CryptoRng, RngCore};
use std::ops::BitAnd;
use subtle::{ConstantTimeEq, ConstantTimeLess, CtOption};

use crate::{Address, OramBlock, BlockValue, CountAccessesDatabase, Database, Oram};

/// A simple ORAM that, for each access, ensures obliviousness by making a complete pass over the database,
/// reading and writing each memory location.
pub struct LinearTimeOram<DB> {
    /// The memory of the ORAM.
    // Made this public for benchmarking, which ideally, I would not need to do.
    pub physical_memory: DB
}

// impl<const B: BlockSize, DB: Database<BlockValue<B>>> Oram<B> for LinearTimeOram<DB> {
impl<V: OramBlock, DB: Database<V>> Oram<V> for LinearTimeOram<DB> where Standard: Distribution<V> {
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, _: &mut R) -> Self {
        Self {
            physical_memory: DB::new(block_capacity),
        }
    }

    // fn block_size(&self) -> BlockSize {
    //     B
    // }

    fn access<R: RngCore + CryptoRng>(
        &mut self,
        index: Address,
        optional_new_value: CtOption<V>,
        _: &mut R,
    ) -> V {
        // Note: index and optional_new_value should be considered secret for the purposes of constant-time operations.

        // TODO(#6): Handle malformed input in a more robust way.
        let index_in_bounds: bool = (index as u128)
            .ct_lt(&(self.block_capacity() as u128))
            .into();

        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not. See also Issue #6.
        assert!(index_in_bounds);

        // This is a dummy value which will always be overwritten.
        let mut result = V::default();

        for i in 0..self.physical_memory.capacity() {
            // Read from memory
            let entry = self.physical_memory.read(i);

            // Client-side processing
            // let is_requested_index: Choice = (u8::from(index == i)).into();
            let is_requested_index = (i as Address).ct_eq(&index);

            // Based on whether the loop counter matches the requested index,
            // conditionally read the value in memory into the result of the access.
            result.conditional_assign(&entry, is_requested_index);

            let oram_operation_is_write = optional_new_value.is_some();
            let should_write = is_requested_index.bitand(oram_operation_is_write);
            // Note that the unwrap_or_else method of CtOption is constant-time.
            let value_to_write = optional_new_value.unwrap_or_else(V::default);

            // Based on whether (1) the loop counter matches the requested index,
            // AND (2) this ORAM access is a write,
            // select the value to be written back out to memory to be either the original value
            // or the provided new value.
            let potentially_updated_value =
                V::conditional_select(&entry, &value_to_write, should_write);
            // End client-side processing

            // Write the (potentially) updated value back to memory.
            self.physical_memory.write(i, potentially_updated_value);
        }
        result
    }

    fn block_capacity(&self) -> Address {
        self.physical_memory.capacity()
    }
}

/// A type alias for a simple `LinearTimeOram` monomorphization.
pub type ConcreteLinearTimeOram<const B: usize> =
    LinearTimeOram<CountAccessesDatabase<BlockValue<B>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::
        SimpleDatabase
    ;
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

    // Block size 64 bytes, block capacity 256 bytes, testing with 10000 operations
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     256,
    //     10000
    // );
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     1,
    //     64,
    //     10000
    // );
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     1,
    //     10000
    // );
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     64,
    //     10000
    // );
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     4096,
    //     64,
    //     1000
    // );
    // create_correctness_test!(
    //     test_correctness_random_workload,
    //     ConcreteLinearTimeOram,
    //     4096,
    //     256,
    //     1000
    // );

    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     256,
    //     100
    // );
    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     1,
    //     64,
    //     100
    // );
    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     1,
    //     100
    // );
    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     64,
    //     64,
    //     100
    // );
    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     4096,
    //     64,
    //     10
    // );
    // create_correctness_test!(
    //     test_correctness_linear_workload,
    //     ConcreteLinearTimeOram,
    //     4096,
    //     256,
    //     2
    // );
}
