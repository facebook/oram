extern crate criterion;
use core::fmt;
use std::fmt::Display;

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion,
};

use oram::{
    BlockValue, CountAccessesDatabase, Database, IndexType, LinearTimeORAM, SimpleDatabase, ORAM,
};
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

// We will benchmark capacities of 64 and 256 for block sizes of 64 and 4096 bytes.
const CAPACITIES_TO_BENCHMARK: [usize; 2] = [64, 256];
const NUM_RANDOM_OPERATIONS_TO_RUN: usize = 64;
const BLOCK_SIZES_TO_BENCHMARK: [usize; 2] = [64, 4096];

criterion_group!(
    benches,
    benchmark_initialization,
    benchmark_read,
    benchmark_write,
    benchmark_random_operations,
    count_accesses,
);
criterion_main!(benches);

fn print_table_row<A: Display, B: Display, C: Display, D: Display>(s1: A, s2: B, s3: C, s4: D) {
    println!(
        "{0: <15} | {1: <15} | {2: <15} | {3: <15}",
        s1, s2, s3, s4
    )
}

fn print_table_header() {
    // println!(
    //     "{0: <15} | {1: <15} | {2: <15} | {3: <15}",
    //     "ORAM Capacity", "ORAM Blocksize", "Physical Reads", "Physical Writes"
    // );
    print_table_row("ORAM Capacity", "ORAM Blocksize", "Physical Reads", "Physical Writes");
}

// fn print_random_accesses_table_header() {
//     println!(
//         "{0: <15} | {1: <15} | {2: <15} | {3: <15} | {4: <15}",
//         "ORAM Capacity", "ORAM Blocksize", "# Operations", "Physical Reads", "Physical Writes"
//     );
// }

fn count_accesses(_: &mut Criterion) {
    println!("Physical reads and writes incurred by 1 ORAM read:");
    print_table_header();
    for capacity in CAPACITIES_TO_BENCHMARK {
        for B in BLOCK_SIZES_TO_BENCHMARK {
            count_accesses_on_read::<{B}>(capacity);
        }
    }

    println!("Physical reads and writes incurred by 1 ORAM write:");
    print_table_header();
    for capacity in CAPACITIES_TO_BENCHMARK {
        count_accesses_on_write(capacity);
    }

    println!("Physical reads and writes incurred by {} random ORAM operations:", NUM_RANDOM_OPERATIONS_TO_RUN);
    print_table_header();
    for capacity in CAPACITIES_TO_BENCHMARK {
        count_accesses_on_random_workload(capacity);
    }
}

fn count_accesses_on_read<const B: usize>(capacity: usize) {
    let mut oram: LinearTimeORAM<CountAccessesDatabase<BlockValue<B>>> =
        LinearTimeORAM::new(capacity);
    oram.read(black_box(0));

    let read_count = oram.physical_memory.get_read_count();
    let write_count= oram.physical_memory.get_write_count();
    // let rwparameters64 = ReadWriteParameters {
    //     capacity: capacity,
    //     block_size: 64,
    // };
    print_table_row(capacity, B, read_count, write_count);
    // println!("A logical read to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters64, read_count64, write_count64);

    // let read_count4096 = oram4096.physical_memory.get_read_count();
    // let write_count4096 = oram4096.physical_memory.get_write_count();
    // let rwparameters4096 = ReadWriteParameters {
    //     capacity: capacity,
    //     block_size: 4096,
    // };
    // print_table_row(capacity, 4096, read_count4096, write_count4096);

    // println!("A logical read to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters4096, read_count4096, write_count4096);
}

// fn count_accesses_on_read(capacity: usize) {
//     let mut oram64: LinearTimeORAM<CountAccessesDatabase<BlockValue<64>>> =
//         LinearTimeORAM::new(capacity);
//     oram64.read(black_box(0));
//     let mut oram4096: LinearTimeORAM<CountAccessesDatabase<BlockValue<4096>>> =
//         LinearTimeORAM::new(capacity);
//     oram4096.read(black_box(0));

//     let read_count64 = oram64.physical_memory.get_read_count();
//     let write_count64 = oram64.physical_memory.get_write_count();
//     // let rwparameters64 = ReadWriteParameters {
//     //     capacity: capacity,
//     //     block_size: 64,
//     // };
//     print_table_row(capacity, 64, read_count64, write_count64);
//     // println!("A logical read to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters64, read_count64, write_count64);

//     let read_count4096 = oram4096.physical_memory.get_read_count();
//     let write_count4096 = oram4096.physical_memory.get_write_count();
//     // let rwparameters4096 = ReadWriteParameters {
//     //     capacity: capacity,
//     //     block_size: 4096,
//     // };
//     print_table_row(capacity, 4096, read_count4096, write_count4096);

//     // println!("A logical read to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters4096, read_count4096, write_count4096);
// }

fn count_accesses_on_write(capacity: usize) {
    let mut oram64: LinearTimeORAM<CountAccessesDatabase<BlockValue<64>>> =
        LinearTimeORAM::new(capacity);
    oram64.write(black_box(0), black_box(BlockValue::default()));
    let mut oram4096: LinearTimeORAM<CountAccessesDatabase<BlockValue<4096>>> =
        LinearTimeORAM::new(capacity);
    oram4096.write(black_box(0), black_box(BlockValue::default()));

    let read_count64 = oram64.physical_memory.get_read_count();
    let write_count64 = oram64.physical_memory.get_write_count();
    let rwparameters64 = ReadWriteParameters {
        capacity: capacity,
        block_size: 64,
    };
    println!("A logical write to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters64, read_count64, write_count64);

    let read_count4096 = oram4096.physical_memory.get_read_count();
    let write_count4096 = oram4096.physical_memory.get_write_count();
    let rwparameters4096 = ReadWriteParameters {
        capacity: capacity,
        block_size: 4096,
    };
    println!("A logical write to ORAM with parameters: {} incurred {} physical reads and {} physical writes.", rwparameters4096, read_count4096, write_count4096);
}

fn count_accesses_on_random_workload(capacity: usize) {
    let number_of_operations_to_run = 64usize;

    let mut rng = StdRng::seed_from_u64(0);

    let mut read_versus_write_randomness = vec![false; number_of_operations_to_run];
    rng.fill(&mut read_versus_write_randomness[..]);
    let mut value_randomness = vec![0u8; 4096 * capacity];
    rng.fill(&mut value_randomness[..]);

    let mut index_randomness = vec![0usize; number_of_operations_to_run];
    for i in 0..number_of_operations_to_run {
        index_randomness[i] = rng.gen_range(0..capacity);
    }

    let mut oram64: LinearTimeORAM<CountAccessesDatabase<BlockValue<64>>> =
        LinearTimeORAM::new(capacity);
    oram64.random_accesses(
        number_of_operations_to_run,
        black_box(&index_randomness),
        black_box(&read_versus_write_randomness),
        black_box(&value_randomness),
    );

    let mut oram4096: LinearTimeORAM<CountAccessesDatabase<BlockValue<4096>>> =
        LinearTimeORAM::new(capacity);
    oram4096.random_accesses(
        number_of_operations_to_run,
        black_box(&index_randomness),
        black_box(&read_versus_write_randomness),
        black_box(&value_randomness),
    );

    let read_count64 = oram64.physical_memory.get_read_count();
    let write_count64 = oram64.physical_memory.get_write_count();
    let rwparameters64 = RandomOperationsParameters {
        capacity: capacity,
        block_size: 64,
        number_of_operations_to_run: number_of_operations_to_run,
    };
    println!("{} random ORAM operations with parameters: {} incurred {} physical reads and {} physical writes.", number_of_operations_to_run, rwparameters64, read_count64, write_count64);

    let read_count4096 = oram4096.physical_memory.get_read_count();
    let write_count4096 = oram4096.physical_memory.get_write_count();
    let rwparameters4096 = RandomOperationsParameters {
        capacity: capacity,
        block_size: 4096,
        number_of_operations_to_run: number_of_operations_to_run,
    };
    println!("{} random ORAM operations with parameters: {} incurred {} physical reads and {} physical writes.", number_of_operations_to_run, rwparameters4096, read_count4096, write_count4096);
}

fn benchmark_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 64,
            }),
            capacity,
            |b, &capacity| {
                b.iter(|| -> LinearTimeORAM<SimpleDatabase<BlockValue<64>>> {
                    LinearTimeORAM::new(capacity)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 4096,
            }),
            capacity,
            |b, &capacity| {
                b.iter(|| -> LinearTimeORAM<SimpleDatabase<BlockValue<4096>>> {
                    LinearTimeORAM::new(capacity)
                })
            },
        );
    }
}

fn benchmark_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram64: LinearTimeORAM<SimpleDatabase<BlockValue<64>>> =
            LinearTimeORAM::new(*capacity);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 64,
            }),
            |b| b.iter(|| oram64.read(0)),
        );
        let mut oram4096: LinearTimeORAM<SimpleDatabase<BlockValue<4096>>> =
            LinearTimeORAM::new(*capacity);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 4096,
            }),
            |b| b.iter(|| oram4096.read(0)),
        );
    }
}

fn benchmark_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");
    for capacity in CAPACITIES_TO_BENCHMARK.iter() {
        let mut oram64: LinearTimeORAM<SimpleDatabase<BlockValue<64>>> =
            LinearTimeORAM::new(*capacity);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 64,
            }),
            |b| b.iter(|| oram64.write(0, BlockValue::default())),
        );
        let mut oram4096: LinearTimeORAM<SimpleDatabase<BlockValue<4096>>> =
            LinearTimeORAM::new(*capacity);
        group.bench_function(
            BenchmarkId::from_parameter(ReadWriteParameters {
                capacity: *capacity,
                block_size: 4096,
            }),
            |b| b.iter(|| oram4096.write(0, BlockValue::default())),
        );
    }
}

fn benchmark_random_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_operations");

    for capacity in CAPACITIES_TO_BENCHMARK {
        let mut oram64: LinearTimeORAM<SimpleDatabase<BlockValue<64>>> =
            LinearTimeORAM::new(capacity);
        let mut oram4096: LinearTimeORAM<SimpleDatabase<BlockValue<4096>>> =
            LinearTimeORAM::new(capacity);

        benchmark_random_operations_helper(&mut oram64, &mut group);
        benchmark_random_operations_helper(&mut oram4096, &mut group);
    }
    group.finish();
}

#[derive(Clone, Copy)]
struct ReadWriteParameters {
    capacity: usize,
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
    capacity: usize,
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

trait CanRunRandomOperations<const B: usize> {
    fn random_accesses(
        &mut self,
        number_of_operations_to_run: usize,
        index_randomness: &[IndexType],
        read_versus_write_randomness: &[bool],
        value_randomness: &[u8],
    ) -> BlockValue<B>;
}

impl<const B: usize, DB: Database<BlockValue<B>>> CanRunRandomOperations<B> for LinearTimeORAM<DB> {
    fn random_accesses(
        &mut self,
        number_of_operations_to_run: usize,
        index_randomness: &[IndexType],
        read_versus_write_randomness: &[bool],
        value_randomness: &[u8],
    ) -> BlockValue<B> {
        for operation_number in 0..number_of_operations_to_run {
            let random_index = index_randomness[operation_number];
            let random_read_versus_write: bool = read_versus_write_randomness[operation_number];

            if random_read_versus_write {
                self.read(random_index);
            } else {
                let block_size = self.block_size();
                let start_index = block_size * random_index;
                let end_index = block_size * (random_index + 1);
                let random_bytes: [u8; B] =
                    value_randomness[start_index..end_index].try_into().unwrap();
                self.write(random_index, BlockValue::from_byte_array(random_bytes));
            }
        }
        BlockValue::default()
    }
}

fn benchmark_random_operations_helper<const B: usize>(
    oram: &mut LinearTimeORAM<SimpleDatabase<BlockValue<B>>>,
    benchmark_group: &mut BenchmarkGroup<WallTime>,
) {
    let number_of_operations_to_run = 64 as usize;

    let block_size = oram.block_size();
    let capacity: usize = oram.block_capacity();
    let parameters = &RandomOperationsParameters {
        capacity,
        block_size,
        number_of_operations_to_run,
    };

    let mut index_randomness = vec![0usize; number_of_operations_to_run];
    let mut read_versus_write_randomness = vec![false; number_of_operations_to_run];
    let mut value_randomness = vec![0u8; block_size * capacity];
    for i in 0..number_of_operations_to_run {
        index_randomness[i] = thread_rng().gen_range(0..capacity);
    }

    thread_rng().fill(&mut read_versus_write_randomness[..]);
    thread_rng().fill(&mut value_randomness[..]);

    benchmark_group.bench_with_input(
        BenchmarkId::from_parameter(parameters),
        parameters,
        |b, &parameters| {
            b.iter(|| {
                oram.random_accesses(
                    parameters.number_of_operations_to_run,
                    black_box(&index_randomness),
                    black_box(&read_versus_write_randomness),
                    black_box(&value_randomness),
                )
            })
        },
    );
}
