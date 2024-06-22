// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! `oram` crate.

use rand::{distributions::{Distribution, Standard}, rngs::StdRng, Rng, SeedableRng};

use crate::{Oram, OramBlock};

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub(crate) fn test_correctness_random_workload<V: OramBlock, T: Oram<V>>(
    capacity: usize,
    num_operations: u32,
) 
where Standard: Distribution<V>
{
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity, &mut rng);
    let mut mirror_array = vec![V::default(); capacity];

    for _ in 0..num_operations {
        let random_index = rng.gen_range(0..capacity);
        let random_block_value = rng.gen::<V>();

        let read_versus_write = rng.gen::<bool>();

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

/// Tests the correctness of an `Oram` type T on repeated passes of sequential accesses 0, 1, ..., `capacity`
pub(crate) fn test_correctness_linear_workload<V: OramBlock, T: Oram<V>>(
    capacity: usize,
    num_passes: u32,
) 
where Standard: Distribution<V>
{
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity, &mut rng);

    let mut mirror_array = vec![V::default(); capacity];

    for _ in 0..num_passes {
        for index in 0..capacity {
            let random_block_value = rng.gen::<V>();

            let read_versus_write: bool = rng.gen::<bool>();

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

macro_rules! create_correctness_test {
    ($function_name:ident, $oram_type: ident, $oram_value_type: ident, $block_capacity:expr, $iterations_to_test: expr) => {
        paste::paste! {
            #[test]
            fn [<$function_name _ $block_capacity _ $oram_value_type:snake _ $iterations_to_test>]() {
                $function_name::<$oram_value_type, $oram_type<$oram_value_type>>($block_capacity, $iterations_to_test);
            }
        }
    };
}

pub(crate) use create_correctness_test;
