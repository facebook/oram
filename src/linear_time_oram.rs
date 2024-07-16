// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple linear-time implementation of Oblivious RAM.

use crate::database::Database;
use crate::{Address, Oram, OramBlock, OramError};
use rand::{CryptoRng, RngCore};
use subtle::{ConstantTimeEq, ConstantTimeLess};

/// A simple ORAM that, for each access, ensures obliviousness by making a complete pass over the database,
/// reading and writing each memory location.
#[derive(Debug)]
pub struct LinearTimeOram<DB> {
    /// The memory of the ORAM (public for benchmarking).
    pub physical_memory: DB,
}

impl<V: OramBlock, DB: Database<V>> Oram<V> for LinearTimeOram<DB> {
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, _: &mut R) -> Result<Self, OramError> {
        Ok(Self {
            physical_memory: DB::new(block_capacity),
        })
    }

    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        _: &mut R,
    ) -> Result<V, OramError> {
        // TODO(#6): Handle malformed input in a more robust way.
        let index_in_bounds: bool = (index as u128)
            .ct_lt(&(self.block_capacity() as u128))
            .into();

        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not. See also Issue #6.
        assert!(index_in_bounds);

        // This is a dummy value which will always be overwritten.
        let mut result = V::default();

        for i in 0..self.block_capacity() {
            let entry = self.physical_memory.read_db(i);

            let is_requested_index = (i as Address).ct_eq(&index);

            result.conditional_assign(&entry, is_requested_index);

            let potentially_updated_value =
                V::conditional_select(&entry, &callback(&entry), is_requested_index);

            self.physical_memory.write_db(i, potentially_updated_value);
        }
        Ok(result)
    }

    fn block_capacity(&self) -> Address {
        self.physical_memory.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        block_value::BlockValue, database::CountAccessesDatabase, test_utils::*, BlockSize,
    };

    type ConcreteLinearTimeOram<const B: BlockSize, V> = LinearTimeOram<CountAccessesDatabase<V>>;

    create_correctness_tests_for_oram_type!(ConcreteLinearTimeOram, BlockValue);
}
