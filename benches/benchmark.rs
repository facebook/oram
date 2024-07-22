// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! This module contains benchmarks for the `oram` crate.

// This allows me to work with a small, fast set of benchmarks (see #31).
#![allow(dead_code)]

extern crate criterion;
use core::fmt;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oram::database::CountAccessesDatabase;
use std::fmt::Display;
use std::time::Duration;

use oram::{Address, Oram};
use oram::{BlockSize, BlockValue};
use rand::{rngs::StdRng, Rng, SeedableRng};

use oram::linear_time_oram::LinearTimeOram;
use oram::path_oram::recursive_secure_path_oram::ConcreteObliviousBlockOram;

const CAPACITIES_TO_BENCHMARK: [Address; 3] = [1 << 14, 1 << 16, 1 << 20];
const NUM_RANDOM_OPERATIONS_TO_RUN: u64 = 64;

trait Instrumented {
    fn get_read_count(&self) -> u64;
    fn get_write_count(&self) -> u64;
    fn short_name() -> String;
}

type BenchmarkLinearTimeOram<const B: BlockSize> =
    LinearTimeOram<CountAccessesDatabase<BlockValue<B>>>;

type BenchmarkRecursiveSecurePathOram<const B: BlockSize> =
    ConcreteObliviousBlockOram<4096, BlockValue<B>>;

impl<const B: BlockSize> Instrumented for BenchmarkRecursiveSecurePathOram<B> {
    fn get_read_count(&self) -> u64 {
        self.physical_memory.get_read_count()
    }

    fn get_write_count(&self) -> u64 {
        self.physical_memory.get_write_count()
    }

    fn short_name() -> String {
        "RecursiveSecureOram".into()
    }
}

impl<const B: BlockSize> Instrumented for BenchmarkLinearTimeOram<B> {
    fn get_read_count(&self) -> u64 {
        return self.physical_memory.get_read_count();
    }

    fn get_write_count(&self) -> u64 {
        return self.physical_memory.get_write_count();
    }

    fn short_name() -> String {
        "LinearTimeOram".into()
    }
}

// Here, all benchmarks are run for linear and path ORAMs, and block sizes of 64 and 4096.
criterion_group!(
    name = benches;
    config = Criterion::default().warm_up_time(Duration::new(0, 1_000_000_00)).measurement_time(Duration::new(0, 1_000_000_00)).sample_size(10);
    targets =
    benchmark_read::<4096, BenchmarkLinearTimeOram<4096>>,
    benchmark_read::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
    benchmark_initialization::<4096, BenchmarkLinearTimeOram<4096>>,
    benchmark_initialization::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
    print_read_header::<BenchmarkLinearTimeOram<0>>,
    count_accesses_on_read::<4096, BenchmarkLinearTimeOram<4096>>,
    print_read_header::<BenchmarkRecursiveSecurePathOram<0>>,
    count_accesses_on_read::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
);
// More comprehensive, slower benchmarks. TODO (#31) this organization should be more formal.
// criterion_group!(
//     name = benches;
//     config = Criterion::default().warm_up_time(Duration::new(0, 1_000_000_00)).measurement_time(Duration::new(0, 1_000_000_00)).sample_size(10);
//     targets =
//     benchmark_read::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     benchmark_initialization::<64, BenchmarkLinearTimeOram<64>>,
//     benchmark_initialization::<4096, BenchmarkLinearTimeOram<4096>>,
//     benchmark_read::<64, BenchmarkLinearTimeOram<64>>,
//     benchmark_read::<4096, BenchmarkLinearTimeOram<4096>>,
//     benchmark_write::<64, BenchmarkLinearTimeOram<64>>,
//     benchmark_write::<4096, BenchmarkLinearTimeOram<4096>>,
//     benchmark_random_operations::<64, BenchmarkLinearTimeOram<64>>,
//     benchmark_random_operations::<4096, BenchmarkLinearTimeOram<4096>>,
//     benchmark_initialization::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     benchmark_initialization::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     benchmark_read::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     benchmark_read::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     benchmark_write::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     benchmark_write::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     benchmark_random_operations::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     benchmark_random_operations::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     print_read_header::<BenchmarkLinearTimeOram<0>>,
//     count_accesses_on_read::<64, BenchmarkLinearTimeOram<64>>,
//     count_accesses_on_read::<4096, BenchmarkLinearTimeOram<4096>>,
//     print_write_header::<BenchmarkLinearTimeOram<0>>,
//     count_accesses_on_write::<64, BenchmarkLinearTimeOram<64>>,
//     count_accesses_on_write::<4096, BenchmarkLinearTimeOram<4096>>,
//     print_random_operations_header::<BenchmarkLinearTimeOram<0>>,
//     count_accesses_on_random_workload::<64, BenchmarkLinearTimeOram<64>>,
//     count_accesses_on_random_workload::<4096, BenchmarkLinearTimeOram<4096>>,
//     print_read_header::<BenchmarkRecursiveSecurePathOram<0>>,
//     count_accesses_on_read::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     count_accesses_on_read::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     print_write_header::<BenchmarkRecursiveSecurePathOram<0>>,
//     count_accesses_on_write::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     count_accesses_on_write::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
//     print_random_operations_header::<BenchmarkRecursiveSecurePathOram<0>>,
//     count_accesses_on_random_workload::<64, BenchmarkRecursiveSecurePathOram<64>>,
//     count_accesses_on_random_workload::<4096, BenchmarkRecursiveSecurePathOram<4096>>,
// );
criterion_main!(benches);

fn count_accesses_on_operation<
    const B: BlockSize,
    T: Oram<BlockValue<B>> + Instrumented,
    F: Fn(&mut T, &mut StdRng, Address) -> (),
>(
    operation: F,
) {
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK {
        let mut oram = T::new(capacity, &mut rng).unwrap();

        let read_count_before = oram.get_read_count();
        let write_count_before = oram.get_write_count();

        operation(&mut oram, &mut rng, capacity);

        let read_count_after = oram.get_read_count();
        let write_count_after = oram.get_write_count();

        let reads_due_to_operation = read_count_after - read_count_before;
        let writes_due_to_operation = write_count_after - write_count_before;

        print_table_row(capacity, B, reads_due_to_operation, writes_due_to_operation);
    }
}

fn count_accesses_on_read<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(
    _: &mut Criterion,
) {
    count_accesses_on_operation(|oram: &mut T, rng, _capacity| {
        oram.read(0, rng).unwrap();
    });
}

fn count_accesses_on_write<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(
    _: &mut Criterion,
) {
    count_accesses_on_operation(|oram: &mut T, rng: &mut StdRng, _capacity| {
        oram.write(0, BlockValue::default(), rng).unwrap();
    });
}

fn count_accesses_on_random_workload<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(
    _: &mut Criterion,
) {
    count_accesses_on_operation(|oram: &mut T, rng, capacity| {
        let number_of_operations_to_run = 64usize;

        let mut read_versus_write_randomness = vec![false; number_of_operations_to_run];
        rng.fill(&mut read_versus_write_randomness[..]);

        let capacity_usize: usize = capacity.try_into().unwrap();
        let mut value_randomness = vec![0u8; 4096 * capacity_usize];
        rng.fill(&mut value_randomness[..]);

        let mut index_randomness = vec![0u64; number_of_operations_to_run];
        for i in 0..number_of_operations_to_run {
            index_randomness[i] = rng.gen_range(0..capacity);
        }

        run_many_random_accesses::<B, T>(
            oram,
            number_of_operations_to_run,
            black_box(&index_randomness),
            black_box(&read_versus_write_randomness),
            black_box(&value_randomness),
        );
    });
}

fn benchmark_initialization<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(T::short_name() + "::initialization");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: B,
            }),
            capacity,
            |b, capacity| b.iter(|| T::new(*capacity, &mut rng)),
        );
    }
}

fn benchmark_read<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(c: &mut Criterion) {
    let mut group = c.benchmark_group(T::short_name() + "::read");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram = T::new(*capacity, &mut rng).unwrap();
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: B,
            }),
            |b| b.iter(|| oram.read(0, &mut rng)),
        );
    }
}

fn benchmark_write<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(c: &mut Criterion) {
    let mut group = c.benchmark_group(T::short_name() + "::write");
    let mut rng = StdRng::seed_from_u64(0);
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram = T::new(*capacity, &mut rng).unwrap();
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: B,
            }),
            |b| b.iter(|| oram.write(0, BlockValue::default(), &mut rng)),
        );
    }
}

fn benchmark_random_operations<const B: usize, T: Oram<BlockValue<B>> + Instrumented>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(T::short_name() + "::random_operations");
    let mut rng = StdRng::seed_from_u64(0);

    for capacity in CAPACITIES_TO_BENCHMARK {
        let mut oram = T::new(capacity, &mut rng).unwrap();

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

fn run_many_random_accesses<const B: usize, T: Oram<BlockValue<B>>>(
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

fn print_table_row<A: Display, B: Display, C: Display, D: Display>(s1: A, s2: B, s3: C, s4: D) {
    println!("{0: <15} | {1: <15} | {2: <15} | {3: <15}", s1, s2, s3, s4)
}

fn print_read_header<T: Instrumented>(_: &mut Criterion) {
    println!("Physical reads and writes incurred by 1 {}::read:", {
        T::short_name()
    });
    print_table_header::<T>();
}

fn print_write_header<T: Instrumented>(_: &mut Criterion) {
    println!();
    println!("Physical reads and writes incurred by 1 {}::write:", {
        T::short_name()
    });
    print_table_header::<T>();
}

fn print_random_operations_header<T: Instrumented>(_: &mut Criterion) {
    println!();
    println!(
        "Physical reads and writes incurred by {} random {} operations:",
        NUM_RANDOM_OPERATIONS_TO_RUN,
        T::short_name()
    );
    print_table_header::<T>();
}
fn print_table_header<T: Instrumented>() {
    print_table_row(
        "ORAM Capacity",
        "ORAM Blocksize",
        "Physical Reads",
        "Physical Writes",
    );
}
