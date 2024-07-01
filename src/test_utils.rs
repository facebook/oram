// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains common test utilities for crates generating tests utilizing the
//! `oram` crate.

use crate::database::{CountAccessesDatabase, Database};
use crate::path_oram::bucket::Bucket;
use crate::path_oram::simple_insecure_path_oram::VecPathOram;
use crate::path_oram::TreeIndex;
use crate::{BucketSize, Oram, OramBlock};
use rand::{
    distributions::{Distribution, Standard},
    rngs::StdRng,
    Rng, SeedableRng,
};
use simplelog::{Config, WriteLogger};
use std::fmt::Debug;
use std::sync::Once;

static INIT: Once = Once::new();

// For use in manual testing and inspection.
#[cfg(test)]
pub(crate) fn init_logger() {
    INIT.call_once(|| {
        WriteLogger::init(
            log::LevelFilter::Info,
            Config::default(),
            std::io::stdout(),
        )
        .unwrap()
    })
}

/// Tests the correctness of an `ORAM` implementation T on a workload of random reads and writes.
pub(crate) fn test_correctness_random_workload<V: OramBlock, T: Oram<V>>(
    capacity: usize,
    num_operations: u32,
) where
    Standard: Distribution<V>,
{
    init_logger();
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
pub(crate) fn test_correctness_linear_workload<V: OramBlock, T: Oram<V> + Debug>(
    capacity: usize,
    num_operations: u32,
) where
    Standard: Distribution<V>,
{
    init_logger();
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
    ($function_name:ident, $oram_type: ident, $block_type: ident, $block_size: expr, $block_capacity:expr, $iterations_to_test: expr) => {
        paste::paste! {
            #[test]
            fn [<$function_name _ $oram_type:snake _ $block_type:snake _ $block_capacity _ $block_size _ $iterations_to_test>]() {
                $function_name::<$block_type<$block_size>, $oram_type<$block_size, $block_type<$block_size>>>($block_capacity, $iterations_to_test);
            }
        }
    };
}

macro_rules! create_correctness_tests_for_workload_and_oram_type {
    ($function_name: ident, $oram_type: ident, $block_type: ident) => {
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 2, 2, 10);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 8, 2, 10);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 16, 2, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 2, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 8, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 16, 16, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 2, 32, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 2, 32, 1000);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 8, 32, 100);
        // Block size 16 bytes, block capacity 32 blocks, testing with 100 operations
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 16, 32, 100);
        create_correctness_test_block_value!($function_name, $oram_type, $block_type, 4, 256, 1000);
    };
}

macro_rules! create_correctness_tests_for_oram_type {
    ($oram_type: ident, $block_type: ident) => {
        create_correctness_tests_for_workload_and_oram_type!(
            test_correctness_linear_workload,
            $oram_type,
            $block_type
        );
        create_correctness_tests_for_workload_and_oram_type!(
            test_correctness_random_workload,
            $oram_type,
            $block_type
        );
    };
}

pub(crate) use create_correctness_test_block_value;
pub(crate) use create_correctness_tests_for_oram_type;
pub(crate) use create_correctness_tests_for_workload_and_oram_type;

macro_rules! monitor_boilerplate {
    () => {
        fn new<R: rand::RngCore + rand::CryptoRng>(
            block_capacity: crate::Address,
            rng: &mut R,
        ) -> Self {
            Self {
                oram: VecPathOram::new(block_capacity, rng),
            }
        }

        fn block_capacity(&self) -> crate::Address {
            self.oram.block_capacity()
        }
    };
}

pub(crate) use monitor_boilerplate;

impl<V: OramBlock, const Z: BucketSize> CountAccessesDatabase<Bucket<V, Z>> {
    fn tree_occupancy(&mut self) -> usize {
        let mut result = 0;
        for i in 0..self.capacity() {
            let bucket = self.read_db(i);
            for block in bucket.blocks {
                if !block.is_dummy() {
                    result += 1;
                }
            }
        }
        result
    }
}

#[derive(Debug)]
pub(crate) struct StashSizeMonitor<T> {
    oram: T,
}

pub(crate) type VecStashSizeMonitor<V, const Z: BucketSize, P> =
    StashSizeMonitor<VecPathOram<V, Z, P>>;

impl<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex> + std::fmt::Debug> Oram<V>
    for VecStashSizeMonitor<V, Z, P>
{
    monitor_boilerplate!();

    fn access<R: rand::RngCore + rand::CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: crate::Address,
        callback: F,
        rng: &mut R,
    ) -> V {
        let result = self.oram.access(index, callback, rng);
        let stash_size = self.oram.stash.len();
        assert!(stash_size < 10);
        result
    }
}

#[derive(Debug)]
pub(crate) struct ConstantOccupancyMonitor<T> {
    oram: T,
}

pub(crate) type VecConstantOccupancyMonitor<V, const Z: BucketSize, P> =
    ConstantOccupancyMonitor<VecPathOram<V, Z, P>>;

// impl <V: OramBlock> Oram<V> for VecConstantOccupancyMonitor<V> {
impl<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex> + std::fmt::Debug> Oram<V>
    for VecConstantOccupancyMonitor<V, Z, P>
{
    monitor_boilerplate!();

    fn access<R: rand::RngCore + rand::CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: crate::Address,
        callback: F,
        rng: &mut R,
    ) -> V {
        let result = self.oram.access(index, callback, rng);

        let mut stash_occupancy = 0;
        for block in &self.oram.stash {
            if !block.is_dummy() {
                stash_occupancy += 1;
            }
        }

        let tree_occupancy = self.oram.physical_memory.tree_occupancy();
        assert_eq!(stash_occupancy + tree_occupancy, self.oram.block_capacity());
        result
    }
}

#[derive(Debug)]
pub(crate) struct PhysicalAccessCountMonitor<T> {
    oram: T,
}

pub(crate) type VecPhysicalAccessCountMonitor<V, const Z: BucketSize, P> =
    PhysicalAccessCountMonitor<VecPathOram<V, Z, P>>;

impl<V: OramBlock, const Z: BucketSize, P: Oram<TreeIndex> + std::fmt::Debug> Oram<V>
    for VecPhysicalAccessCountMonitor<V, Z, P>
{
    monitor_boilerplate!();

    fn access<R: rand::RngCore + rand::CryptoRng, F: Fn(&V) -> V>(
        &mut self,
        index: crate::Address,
        callback: F,
        rng: &mut R,
    ) -> V {
        let pre_read_count = self.oram.physical_memory.get_read_count();
        let pre_write_count = self.oram.physical_memory.get_write_count();

        let result = self.oram.access(index, callback, rng);

        let post_read_count = self.oram.physical_memory.get_read_count();
        let post_write_count = self.oram.physical_memory.get_write_count();

        let reads = post_read_count - pre_read_count;
        let writes = post_write_count - pre_write_count;

        assert_eq!(reads, self.oram.block_capacity().ilog2() as u128);
        assert_eq!(writes, self.oram.block_capacity().ilog2() as u128);

        result
    }
}

macro_rules! create_statistics_test {
    ($function_name:ident, $oram_type: ident, $block_type: ident, $block_size: expr, $block_capacity:expr, $iterations_to_test: expr) => {
        paste::paste! {
            #[test]
            fn [<$function_name _ $oram_type:snake _ $block_capacity _ $block_size _ $iterations_to_test>]() {
                $function_name::<$block_type<$block_size>, $oram_type<$block_size, $block_type<$block_size>>>($block_capacity, $iterations_to_test);
            }
        }
    };
}

macro_rules! create_statistics_test_for_workload_and_oram_type {
    ($function_name: ident, $oram_type: ident, $block_type: ident) => {
        create_statistics_test!($function_name, $oram_type, $block_type, 2, 32, 1000);
    };
}

macro_rules! create_statistics_test_for_oram_type {
    ($oram_type: ident, $block_type: ident) => {
        impl<const B: BlockSize, V: OramBlock> Drop for $oram_type<B, V> {
            fn drop(&mut self) {
                let reads = &self.oram.physical_memory.reads;
                let writes: &Vec<u128> = &self.oram.physical_memory.writes;

                for (r, w) in zip(reads, writes) {
                    assert_eq!(*r, *w);
                }

                let first_leaf_index = 2u64.pow(self.oram.height);
                let last_leaf_index = first_leaf_index * 2 - 1;
                let num_leaves = last_leaf_index - first_leaf_index;

                let mut total_reads = 0;
                for leaf in first_leaf_index..last_leaf_index {
                    total_reads += reads[leaf as usize];
                }

                let expected_reads_per_leaf: u128 = total_reads / num_leaves as u128;

                for leaf in first_leaf_index..=last_leaf_index {
                    assert!(
                        reads[leaf as usize]
                            > expected_reads_per_leaf - expected_reads_per_leaf / 2
                    );
                    assert!(
                        reads[leaf as usize]
                            < expected_reads_per_leaf + expected_reads_per_leaf / 2
                    );
                }
            }
        }

        impl<const B: BlockSize, V: OramBlock> Oram<V> for $oram_type<B, V> {
            fn new<R: rand::RngCore + rand::CryptoRng>(
                block_capacity: crate::Address,
                rng: &mut R,
            ) -> Self {
                let mut oram = VecPathOram::new(block_capacity, rng);

                // Avoid counting reads and writes occurring during initialization
                for i in 0..oram.physical_memory.reads.len() {
                    oram.physical_memory.reads[i] = 0;
                    oram.physical_memory.writes[i] = 0;
                }

                Self { oram: oram }
            }

            fn block_capacity(&self) -> crate::Address {
                self.oram.block_capacity()
            }

            fn access<R: rand::RngCore + rand::CryptoRng, F: Fn(&V) -> V>(
                &mut self,
                index: crate::Address,
                callback: F,
                rng: &mut R,
            ) -> V {
                self.oram.access(index, callback, rng)
            }
        }

        create_statistics_test_for_workload_and_oram_type!(
            test_correctness_linear_workload,
            $oram_type,
            $block_type
        );
        create_statistics_test_for_workload_and_oram_type!(
            test_correctness_random_workload,
            $oram_type,
            $block_type
        );
    };
}

pub(crate) use create_statistics_test;
pub(crate) use create_statistics_test_for_oram_type;
pub(crate) use create_statistics_test_for_workload_and_oram_type;
