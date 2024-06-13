// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! ORAM crate

#![allow(clippy::needless_range_loop)]

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{BlockValue, CountAccessesDatabase, LinearTimeORAM, ORAM};

/// A type alias for the `LinearTimeOram` monomorphization used in testing, to improve readability.
pub type LinearORAM<const B: usize> = LinearTimeORAM<CountAccessesDatabase<BlockValue<B>>>;

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub fn test_correctness_random_workload<const B: usize, T: ORAM<B>>(
    capacity: usize,
    num_operations: u32,
) {
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity);
    let mut mirror_array = vec![BlockValue::default(); capacity];

    for _ in 0..num_operations {
        let random_index = rng.gen_range(0..capacity);
        let random_block_value = rng.gen();

        let read_versus_write: bool = rng.gen();

        if read_versus_write {
            assert_eq!(oram.read(random_index), mirror_array[random_index]);
        } else {
            oram.write(random_index, random_block_value);
            mirror_array[random_index] = random_block_value;
        }
    }

    for index in 0..capacity {
        assert_eq!(oram.read(index), mirror_array[index], "{index}")
    }
}

/// Tests the correctness of an ORAM type T on repeated passes of sequential accesses 0, 1, ..., `capacity`
pub fn test_correctness_linear_workload<const B: usize, T: ORAM<B>>(
    capacity: usize,
    num_passes: u32,
) {
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity);

    let mut mirror_array = vec![BlockValue::default(); capacity];

    for _ in 0..num_passes {
        for index in 0..capacity {
            let random_block_value = rng.gen();

            let read_versus_write: bool = rng.gen();

            if read_versus_write {
                assert_eq!(oram.read(index), mirror_array[index]);
            } else {
                oram.write(index, random_block_value);
                mirror_array[index] = random_block_value;
            }
        }
    }

    for index in 0..capacity {
        assert_eq!(oram.read(index), mirror_array[index], "{index}")
    }
}
