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
use crate::{
    Address, BlockSize, BucketSize, Oram, OramBlock, OramError, RecursionCutoff, StashSize,
};
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng, SeedableRng,
};
use simplelog::{Config, WriteLogger};

// For use in manual testing and inspection.
// Change log_level to "Warn" to see stash overflow events, and to "Debug" to additionally see ORAM initialization events.
pub(crate) fn init_logger() {
    INIT.call_once(|| {
        WriteLogger::init(
            log::LevelFilter::Error,
            Config::default(),
            std::io::stdout(),
        )
        .unwrap()
    })
}

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub(crate) fn random_workload<T: Oram>(oram: &mut T, num_operations: usize)
where
    Standard: Distribution<T::V>,
{
    init_logger();
    let mut rng = StdRng::seed_from_u64(0);

    let capacity = oram.block_capacity().unwrap();
    let mut mirror_array = vec![T::V::default(); usize::try_from(capacity).unwrap()];

    for _ in 0..num_operations {
        let random_index = rng.gen_range(0..capacity);
        let random_block_value = rng.gen::<T::V>();

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
pub(crate) fn linear_workload<T: Oram + Debug>(oram: &mut T, num_operations: u64)
where
    Standard: Distribution<T::V>,
{
    init_logger();
    let mut rng = StdRng::seed_from_u64(0);

    let capacity = oram.block_capacity().unwrap();
    let mut mirror_array = vec![T::V::default(); usize::try_from(capacity).unwrap()];

    let num_passes = num_operations / capacity;

    for _ in 0..num_passes {
        for index in 0..capacity {
            let random_block_value = rng.gen::<T::V>();

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

macro_rules! create_path_oram_correctness_tests_all_parameters {
    ($oram_type: ident, $prefix: literal, $block_capacity: expr, $block_size: expr, $bucket_size: expr, $position_block_size: expr, $overflow_size: expr, $recursion_cutoff: expr, $iterations_to_test: expr) => {
        paste::paste! {
            #[test]
            fn [<"linear_workload" $prefix $block_capacity _ $block_size _ $bucket_size _ $position_block_size _ $overflow_size _ $recursion_cutoff>]() {
                let mut rng = StdRng::seed_from_u64(1);
                let mut oram = $oram_type::<BlockValue<$block_size>, $bucket_size, $position_block_size>::new_with_parameters($block_capacity, &mut rng, $overflow_size, $recursion_cutoff).unwrap();
                linear_workload(&mut oram, $iterations_to_test);
            }

            #[test]
            fn [<"random_workload" $prefix $block_capacity _ $block_size _ $bucket_size _ $position_block_size _ $overflow_size _ $recursion_cutoff>]() {
                let mut rng = StdRng::seed_from_u64(1);
                let mut oram = $oram_type::<BlockValue<$block_size>, $bucket_size, $position_block_size>::new_with_parameters($block_capacity, &mut rng, $overflow_size, $recursion_cutoff).unwrap();
                random_workload(&mut oram, $iterations_to_test);
            }
        }
    };
}

macro_rules! create_path_oram_correctness_tests_helper {
    ($oram_type: ident, $prefix: literal, $bucket_size: expr, $position_block_size: expr, $recursion_cutoff: expr, $overflow_size: expr) => {
        create_path_oram_correctness_tests_all_parameters!(
            $oram_type,
            $prefix,
            8,
            1,
            $bucket_size,
            $position_block_size,
            $overflow_size,
            $recursion_cutoff,
            100
        );
        create_path_oram_correctness_tests_all_parameters!(
            $oram_type,
            $prefix,
            4,
            1,
            $bucket_size,
            $position_block_size,
            $overflow_size,
            $recursion_cutoff,
            100
        );
        // Block size 4 blocks, block size 2 bytes, testing with 100 operations
        create_path_oram_correctness_tests_all_parameters!(
            $oram_type,
            $prefix,
            4,
            2,
            $bucket_size,
            $position_block_size,
            $overflow_size,
            $recursion_cutoff,
            100
        );
        create_path_oram_correctness_tests_all_parameters!(
            $oram_type,
            $prefix,
            16,
            1,
            $bucket_size,
            $position_block_size,
            $overflow_size,
            $recursion_cutoff,
            100
        );
        create_path_oram_correctness_tests_all_parameters!(
            $oram_type,
            $prefix,
            2,
            1,
            $bucket_size,
            $position_block_size,
            $overflow_size,
            $recursion_cutoff,
            1000
        );
    };
}

macro_rules! create_path_oram_correctness_tests {
    ($bucket_size: expr, $position_block_size: expr, $recursion_cutoff: expr, $overflow_size: expr) => {
        create_path_oram_correctness_tests_helper!(
            PathOram,
            "",
            $bucket_size,
            $position_block_size,
            $recursion_cutoff,
            $overflow_size
        );
    };
}

macro_rules! create_path_oram_stash_size_tests {
    ($bucket_size: expr, $position_block_size: expr, $recursion_cutoff: expr, $overflow_size: expr) => {
        create_path_oram_correctness_tests_helper!(
            StashSizeMonitor,
            "_stash_size_",
            $bucket_size,
            $position_block_size,
            $recursion_cutoff,
            $overflow_size
        );
    };
}

#[derive(Debug)]
pub(crate) struct StashSizeMonitor<V: OramBlock, const Z: BucketSize, const AB: BlockSize> {
    oram: PathOram<V, Z, AB>,
}

impl<V: OramBlock, const Z: BucketSize, const AB: BlockSize> StashSizeMonitor<V, Z, AB> {
    pub(crate) fn new_with_parameters<R: rand::RngCore + rand::CryptoRng>(
        block_capacity: Address,
        rng: &mut R,
        overflow_size: StashSize,
        recursion_cutoff: RecursionCutoff,
    ) -> Result<Self, OramError> {
        Ok(Self {
            oram: PathOram::new_with_parameters(
                block_capacity,
                rng,
                overflow_size,
                recursion_cutoff,
            )
            .unwrap(),
        })
    }
}

impl<V: OramBlock, const Z: BucketSize, const AB: BlockSize> Oram for StashSizeMonitor<V, Z, AB> {
    type V = V;

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
        let stash_size = self.oram.stash_occupancy();
        assert!(stash_size < 10);
        result
    }
}

pub(crate) use create_path_oram_correctness_tests;
pub(crate) use create_path_oram_correctness_tests_all_parameters;
pub(crate) use create_path_oram_correctness_tests_helper;
pub(crate) use create_path_oram_stash_size_tests;
