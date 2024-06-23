// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! `oram` crate.

use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    CryptoRng, Rng, RngCore, SeedableRng,
};

use duplicate::duplicate_item;

use crate::{
    linear_time_oram::ConcreteLinearTimeOram,
    simple_insecure_path_oram::ConcreteSimpleInsecurePathOram, Address, CountAccessesDatabase,
    Database, Oram, OramBlock, SimpleDatabase,
};
pub(crate) trait TestableMemory<V: OramBlock> {
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, rng: &mut R) -> Self;

    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> V;

    fn write<R: RngCore + CryptoRng>(&mut self, index: Address, new_value: V, rng: &mut R);
}

#[duplicate_item(
    oram_type;
    [ConcreteLinearTimeOram];
    [ConcreteSimpleInsecurePathOram];
)]
impl<V: OramBlock> TestableMemory<V> for oram_type<V>
where
    Standard: Distribution<V>,
{
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, rng: &mut R) -> Self {
        Oram::new(block_capacity, rng)
    }

    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, rng: &mut R) -> V {
        Oram::read(self, index, rng)
    }

    fn write<R: RngCore + CryptoRng>(&mut self, index: Address, new_value: V, rng: &mut R) {
        Oram::write(self, index, new_value, rng)
    }
}

#[duplicate_item(
    database_type;
    [SimpleDatabase];
    [CountAccessesDatabase];
)]
impl<V: OramBlock> TestableMemory<V> for database_type<V> {
    fn new<R: RngCore + CryptoRng>(block_capacity: Address, _: &mut R) -> Self {
        Database::new(block_capacity)
    }

    fn read<R: RngCore + CryptoRng>(&mut self, index: Address, _: &mut R) -> V {
        Database::read(self, index)
    }

    fn write<R: RngCore + CryptoRng>(&mut self, index: Address, new_value: V, _: &mut R) {
        Database::write(self, index, new_value)
    }
}

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub(crate) fn test_correctness_random_workload<V: OramBlock, T: TestableMemory<V>>(
    capacity: usize,
    num_operations: u32,
) where
    Standard: Distribution<V>,
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
pub(crate) fn test_correctness_linear_workload<V: OramBlock, T: TestableMemory<V>>(
    capacity: usize,
    num_operations: u32,
) where
    Standard: Distribution<V>,
{
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity, &mut rng);

    let mut mirror_array = vec![V::default(); capacity];

    let num_passes = (num_operations as usize) / capacity;

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

macro_rules! create_correctness_test_block_value {
    ($function_name:ident, $oram_type: ident, $block_size: expr, $block_capacity:expr, $iterations_to_test: expr) => {
        paste::paste! {
            #[test]
            fn [<$function_name _ $oram_type:snake _ $block_capacity _ $block_size _ $iterations_to_test>]() {
                $function_name::<BlockValue<$block_size>, $oram_type<BlockValue<$block_size>>>($block_capacity, $iterations_to_test);
            }
        }
    };
}

macro_rules! create_correctness_tests_for_workload_and_oram_type {
    ($function_name: ident, $oram_type: ident) => {
        create_correctness_test_block_value!($function_name, $oram_type, 1, 2, 10);
        create_correctness_test_block_value!($function_name, $oram_type, 8, 2, 10);
        create_correctness_test_block_value!($function_name, $oram_type, 16, 2, 100);
        create_correctness_test_block_value!($function_name, $oram_type, 1, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, 8, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, 16, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, 1, 32, 100);
        create_correctness_test_block_value!($function_name, $oram_type, 1, 32, 1000);
        create_correctness_test_block_value!($function_name, $oram_type, 8, 32, 100);
        // Block size 16 bytes, block capacity 32 blocks, testing with 100 operations
        create_correctness_test_block_value!($function_name, $oram_type, 16, 32, 100);
    };
}

macro_rules! create_correctness_tests_for_oram_type {
    ($oram_type: ident) => {
        create_correctness_tests_for_workload_and_oram_type!(
            test_correctness_linear_workload,
            $oram_type
        );
        create_correctness_tests_for_workload_and_oram_type!(
            test_correctness_random_workload,
            $oram_type
        );
    };
}

pub(crate) use create_correctness_test_block_value;
pub(crate) use create_correctness_tests_for_oram_type;
pub(crate) use create_correctness_tests_for_workload_and_oram_type;
