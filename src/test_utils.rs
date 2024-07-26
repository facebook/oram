// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! `oram` crate.

use std::fmt::Debug;
use std::sync::Once;
static INIT: Once = Once::new();
use crate::path_oram::PathOram;
use crate::StashSize;
use crate::{Address, BlockSize, BucketSize, Oram, OramBlock, OramError, RecursionCutoff};
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng, SeedableRng,
};
use simplelog::{Config, WriteLogger};

// For use in manual testing and inspection.
pub(crate) fn init_logger() {
    INIT.call_once(|| {
        WriteLogger::init(log::LevelFilter::Info, Config::default(), std::io::stdout()).unwrap()
    })
}

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub(crate) fn random_workload<V: OramBlock, T: Oram<V>>(capacity: Address, num_operations: usize)
where
    Standard: Distribution<V>,
{
    init_logger();
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity, &mut rng).unwrap();
    let mut mirror_array = vec![V::default(); usize::try_from(capacity).unwrap()];

    for _ in 0..num_operations {
        let random_index = rng.gen_range(0..capacity);
        let random_block_value = rng.gen::<V>();

        let read_versus_write = rng.gen::<bool>();

        if read_versus_write {
            assert_eq!(
                oram.read(random_index, &mut rng).unwrap(),
                mirror_array[usize::try_from(random_index).unwrap()]
            );
        } else {
            oram.write(random_index, random_block_value, &mut rng)
                .unwrap();
            mirror_array[usize::try_from(random_index).unwrap()] = random_block_value;
        }
    }

    for index in 0..capacity {
        assert_eq!(
            oram.read(index, &mut rng).unwrap(),
            mirror_array[usize::try_from(index).unwrap()],
            "{index}"
        )
    }
}

/// Tests the correctness of an `Oram` type T on repeated passes of sequential accesses 0, 1, ..., `capacity`
pub(crate) fn linear_workload<V: OramBlock, T: Oram<V> + Debug>(
    capacity: Address,
    num_operations: u64,
) where
    Standard: Distribution<V>,
{
    init_logger();
    let mut rng = StdRng::seed_from_u64(0);

    let mut oram = T::new(capacity, &mut rng).unwrap();
    let mut mirror_array = vec![V::default(); usize::try_from(capacity).unwrap()];

    let num_passes = num_operations / capacity;

    for _ in 0..num_passes {
        for index in 0..capacity {
            let random_block_value = rng.gen::<V>();

            let read_versus_write: bool = rng.gen::<bool>();

            if read_versus_write {
                assert_eq!(
                    oram.read(index, &mut rng).unwrap(),
                    mirror_array[usize::try_from(index).unwrap()]
                );
            } else {
                oram.write(index, random_block_value, &mut rng).unwrap();
                mirror_array[usize::try_from(index).unwrap()] = random_block_value;
            }
        }
    }

    for index in 0..capacity {
        assert_eq!(
            oram.read(index, &mut rng).unwrap(),
            mirror_array[usize::try_from(index).unwrap()],
            "{index}"
        )
    }
}

macro_rules! create_correctness_test {
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
        create_correctness_test!($function_name, $oram_type, 2, 2, 10);
        create_correctness_test!($function_name, $oram_type, 4, 8, 100);
        create_correctness_test!($function_name, $oram_type, 2, 8, 100);
        create_correctness_test!($function_name, $oram_type, 8, 8, 100);
        create_correctness_test!($function_name, $oram_type, 4, 16, 100);
        create_correctness_test!($function_name, $oram_type, 4, 32, 100);
        // Block size 16 bytes, block capacity 64 blocks, testing with 100 operations
        create_correctness_test!($function_name, $oram_type, 16, 64, 100);
        create_correctness_test!($function_name, $oram_type, 2, 8, 1000);
    };
}

macro_rules! create_correctness_tests_for_oram_type {
    ($oram_type: ident) => {
        create_correctness_tests_for_workload_and_oram_type!(linear_workload, $oram_type);
        create_correctness_tests_for_workload_and_oram_type!(random_workload, $oram_type);
    };
}

macro_rules! create_correctness_tests_for_path_oram {
    ($bucket_size: expr, $position_block_size: expr, $recursion_cutoff: expr, $stash_overflow_size: expr) => {
        paste::paste! {
            type [<PathOram $bucket_size _ $position_block_size _ $recursion_cutoff _ $stash_overflow_size>]<V> = PathOram<V, $bucket_size, $position_block_size, $recursion_cutoff, $stash_overflow_size>;
            create_correctness_tests_for_oram_type!([<PathOram $bucket_size _ $position_block_size _ $recursion_cutoff _ $stash_overflow_size>]);
        }
    };
}

#[derive(Debug)]
pub(crate) struct StashSizeMonitor<T> {
    oram: T,
}

impl<
        V: OramBlock,
        const Z: BucketSize,
        const AB: BlockSize,
        const RT: RecursionCutoff,
        const SO: StashSize,
    > Oram<V> for StashSizeMonitor<PathOram<V, Z, AB, RT, SO>>
{
    fn new<R: rand::RngCore + rand::CryptoRng>(
        block_capacity: Address,
        rng: &mut R,
    ) -> Result<Self, OramError> {
        Ok(Self {
            oram: PathOram::new(block_capacity, rng)?,
        })
    }

    fn block_capacity(&self) -> Result<Address, OramError> {
        self.oram.block_capacity()
    }

    fn access<R: rand::RngCore + rand::CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: Address,
        callback: F,
        rng: &mut R,
    ) -> Result<V, OramError> {
        let result = self.oram.access(index, callback, rng);
        let stash_size = self.oram.stash.occupancy();
        assert!(stash_size < 10);
        result
    }
}

macro_rules! create_stash_size_tests {
    ($oram_type: ident) => {
        paste::paste! {
            type [<MonitorStashSize $oram_type>]<V> = StashSizeMonitor<$oram_type<V>>;
            create_correctness_tests_for_oram_type!([<MonitorStashSize $oram_type>]);
        }
    };
}

pub(crate) use create_correctness_test;
pub(crate) use create_correctness_tests_for_oram_type;
pub(crate) use create_correctness_tests_for_path_oram;
pub(crate) use create_correctness_tests_for_workload_and_oram_type;
pub(crate) use create_stash_size_tests;
