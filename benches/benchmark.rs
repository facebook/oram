// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains benchmarks for the `oram` crate.

extern crate criterion;
use core::fmt;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oram::DefaultOram;
use rand::CryptoRng;
use rand::RngCore;
use std::mem;
use std::time::Duration;

use oram::BlockSize;
use oram::BlockValue;
use oram::{Address, Oram};
use rand::{rngs::StdRng, Rng, SeedableRng};

const CAPACITIES_TO_BENCHMARK: [Address; 3] = [1 << 14, 1 << 16, 1 << 20];

trait Benchmarkable {
    fn short_name() -> String;
    fn new<R: CryptoRng + RngCore>(capacity: Address, rng: &mut R) -> Self;
}

impl<const B: BlockSize> Benchmarkable for DefaultOram<BlockValue<B>> {
    fn short_name() -> String {
        "DefaultOram".into()
    }

    fn new<R: CryptoRng + RngCore>(capacity: Address, rng: &mut R) -> Self {
        Self::new(capacity, rng).unwrap()
    }
}

// Here, all benchmarks are run for linear and path ORAMs, and block sizes of 64 and 4096.
criterion_group!(
    name = benches;
    config = Criterion::default().warm_up_time(Duration::new(0, 1_000_000_00)).measurement_time(Duration::new(0, 1_000_000_00)).sample_size(10);
    targets =
    benchmark_read::<DefaultOram<BlockValue<4096>>>,
    benchmark_write::<DefaultOram<BlockValue<4096>>>,
    benchmark_initialization::<DefaultOram<BlockValue<4096>>>,
    benchmark_random_operations::<4096, DefaultOram<BlockValue<4096>>>,
    benchmark_read::<DefaultOram<BlockValue<64>>>,
    benchmark_write::<DefaultOram<BlockValue<64>>>,
    benchmark_initialization::<DefaultOram<BlockValue<64>>>,
    benchmark_random_operations::<64, DefaultOram<BlockValue<64>>>,
);

criterion_main!(benches);

fn benchmark_initialization<T: Oram + Benchmarkable>(c: &mut Criterion) {
    let mut group = c.benchmark_group(T::short_name() + "::initialization");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: mem::size_of::<T::V>(),
            }),
            capacity,
            |b, capacity| b.iter(|| T::new(*capacity, &mut rng)),
        );
    }
}

fn benchmark_read<T: Oram + Benchmarkable>(c: &mut Criterion) {
    let mut group = c.benchmark_group(T::short_name() + "::read");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram = T::new(*capacity, &mut rng);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: mem::size_of::<T::V>(),
            }),
            |b| b.iter(|| oram.read(0, &mut rng)),
        );
    }
}

fn benchmark_write<T: Oram + Benchmarkable>(c: &mut Criterion) {
    let mut group = c.benchmark_group(T::short_name() + "::write");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram = T::new(*capacity, &mut rng);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: mem::size_of::<T::V>(),
            }),
            |b| b.iter(|| oram.write(0, T::V::default(), &mut rng)),
        );
    }
}

fn benchmark_random_operations<const B: BlockSize, T: Oram<V = BlockValue<B>> + Benchmarkable>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(T::short_name() + "::random_operations");
    let mut rng = StdRng::seed_from_u64(0);

    for capacity in CAPACITIES_TO_BENCHMARK {
        let mut oram = T::new(capacity, &mut rng);

        let number_of_operations_to_run = 64 as usize;

        let block_size = B;
        let capacity = oram.block_capacity().unwrap();
        let parameters = &RandomOperationsParameters {
            capacity,
            block_size,
            number_of_operations_to_run,
        };

        let mut index_randomness = vec![0u64; number_of_operations_to_run];
        let mut read_versus_write_randomness = vec![false; number_of_operations_to_run];
        let capacity_usize: usize = capacity.try_into().unwrap();
        let mut value_randomness = vec![0u8; block_size * capacity_usize];
        for i in 0..number_of_operations_to_run {
            index_randomness[i] = rng.gen_range(0..capacity);
        }

        rng.fill(&mut read_versus_write_randomness[..]);
        rng.fill(&mut value_randomness[..]);

        group.bench_with_input(
            BenchmarkId::from_parameter(parameters),
            parameters,
            |b, &parameters| {
                b.iter(|| {
                    run_many_random_accesses::<B, T>(
                        &mut oram,
                        parameters.number_of_operations_to_run,
                        black_box(&index_randomness),
                        black_box(&read_versus_write_randomness),
                        black_box(&value_randomness),
                    )
                })
            },
        );
    }
    group.finish();
}

fn run_many_random_accesses<const B: BlockSize, T: Oram<V = BlockValue<B>>>(
    oram: &mut T,
    number_of_operations_to_run: usize,
    index_randomness: &[Address],
    read_versus_write_randomness: &[bool],
    value_randomness: &[u8],
) -> BlockValue<B> {
    let mut rng = StdRng::seed_from_u64(0);
    for operation_number in 0..number_of_operations_to_run {
        let random_index = index_randomness[operation_number];
        let random_read_versus_write: bool = read_versus_write_randomness[operation_number];

        if random_read_versus_write {
            oram.read(random_index, &mut rng).unwrap();
        } else {
            let block_size = B;
            let random_index_usize: usize = random_index.try_into().unwrap();
            let start_index = block_size * random_index_usize;
            let end_index = block_size * random_index_usize;
            let random_bytes: [u8; B] =
                value_randomness[start_index..end_index].try_into().unwrap();
            oram.write(random_index, BlockValue::new(random_bytes), &mut rng)
                .unwrap();
        }
    }

    BlockValue::default()
}

#[derive(Clone, Copy)]
struct ReadWriteParameters {
    capacity: Address,
    block_size: usize,
}

impl fmt::Display for ReadWriteParameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Capacity: {} Blocksize: {})",
            self.capacity, self.block_size,
        )
    }
}

#[derive(Clone, Copy)]
struct RandomOperationsParameters {
    capacity: Address,
    block_size: usize,
    number_of_operations_to_run: usize,
}

impl fmt::Display for RandomOperationsParameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Capacity: {} Blocksize: {}, Ops: {})",
            self.capacity, self.block_size, self.number_of_operations_to_run,
        )
    }
}
