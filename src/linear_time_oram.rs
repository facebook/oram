// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! A simple linear-time implementation of Oblivious RAM.

use crate::{Address, Oram, OramBlock, OramError};
use rand::{CryptoRng, RngCore};
use subtle::{ConstantTimeEq, ConstantTimeLess};

/// A simple ORAM that, for each access, ensures obliviousness by making a complete pass over the database,
/// reading and writing each memory location.
#[derive(Debug)]
pub struct LinearTimeOram<V: OramBlock> {
    /// The memory of the ORAM (public for benchmarking).
    pub physical_memory: Vec<V>,
}

impl<V: OramBlock> Oram<V> for LinearTimeOram<V> {
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, _: &mut R) -> Result<Self, OramError> {
        let mut physical_memory = Vec::new();
        physical_memory.resize(usize::try_from(block_capacity)?, V::default());
        Ok(Self { physical_memory })
    }

    fn access<R: RngCore + CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        _: &mut R,
    ) -> Result<V, OramError> {
        let index_in_bounds: bool = index.ct_lt(&self.block_capacity()?).into();

        // This operation is not constant-time, but only leaks whether the ORAM index is well-formed or not.
        if !index_in_bounds {
            return Err(OramError::AddressOutOfBoundsError);
        }

        // This is a dummy value which will always be overwritten.
        let mut result = V::default();

        for i in 0..self.physical_memory.len() {
            let entry = &self.physical_memory[i];

            let is_requested_index = (u64::try_from(i)?).ct_eq(&index);

            result.conditional_assign(entry, is_requested_index);

            let potential_new_value = callback(entry);

            self.physical_memory[i].conditional_assign(&potential_new_value, is_requested_index);
        }
        Ok(result)
    }

    fn block_capacity(&self) -> Result<Address, OramError> {
        Ok(u64::try_from(self.physical_memory.len())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bucket::BlockValue, test_utils::*};

    create_correctness_tests_for_oram_type!(LinearTimeOram);
}
