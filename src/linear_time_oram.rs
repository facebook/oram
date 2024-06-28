// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple linear-time implementation of Oblivious RAM.

use rand::{
    distributions::{Distribution, Standard},
    CryptoRng, RngCore,
};

use std::ops::BitAnd;
use subtle::{ConstantTimeEq, ConstantTimeLess};

use crate::{Address, CountAccessesDatabase, Database, Oram, OramBlock};

/// A simple ORAM that, for each access, ensures obliviousness by making a complete pass over the database,
/// reading and writing each memory location.
#[derive(Debug)]
pub struct LinearTimeOram<DB> {
    /// The memory of the ORAM.
    // Made this public for benchmarking, which ideally, I would not need to do.
    pub physical_memory: DB,
}

impl<V: OramBlock, DB: Database<V>> Oram<V> for LinearTimeOram<DB>
where
    Standard: Distribution<V>,
{
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, _: &mut R) -> Self {
        Self {
            physical_memory: DB::new(block_capacity),
        }
    }

    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
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

            let is_requested_index = (i as Address).ct_eq(&index);

            result.conditional_assign(&entry, is_requested_index);

            let potentially_updated_value =
                V::conditional_select(&entry, &callback(&entry), is_requested_index);

            self.physical_memory.write(i, potentially_updated_value);
        }
        result
    }

    fn block_capacity(&self) -> Address {
        self.physical_memory.capacity()
    }
}

/// A type alias for a simple `LinearTimeOram` monomorphization.
pub type ConcreteLinearTimeOram<V> = LinearTimeOram<CountAccessesDatabase<V>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::{
            create_correctness_test_block_value, create_correctness_tests_for_oram_type,
            create_correctness_tests_for_workload_and_oram_type, test_correctness_linear_workload,
            test_correctness_random_workload,
        },
        BlockValue, SimpleDatabase,
    };
    use std::mem;

    #[test]
    fn check_alignment() {
        let irrelevant_capacity = 64;
        let expected_alignment = 64;
        let database =
            <SimpleDatabase<BlockValue<64>> as Database<BlockValue<64>>>::new(irrelevant_capacity);
        for block in &database.0 {
            assert_eq!(mem::align_of_val(block), expected_alignment);
        }
    }
    create_correctness_tests_for_oram_type!(ConcreteLinearTimeOram);
}
