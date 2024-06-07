# Running benchmarks

Use `cargo bench` to run benchmarks.

# Example benchmark output

```
spencerpeters@spencerpeters-mbp oram % cargo bench
   Compiling oram v0.1.0 (/Users/spencerpeters/oram)
    Finished `bench` profile [optimized] target(s) in 3.75s
     Running unittests src/lib.rs (target/release/deps/oram-d78a20f5a591c3c1)

running 3 tests
test tests::check_alignment ... ignored
test tests::check_correctness ... ignored
test tests::simple_read_write ... ignored

test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/benchmark.rs (target/release/deps/benchmark-77f60dec7edfd1fb)
Benchmarking initialization/(Capacity: 64 Blocksize: 64): Collecting 100 samples in estimated 5.0058 s (2.2M i
initialization/(Capacity: 64 Blocksize: 64)
                        time:   [2.2315 µs 2.2346 µs 2.2380 µs]
                        change: [-2.1784% -1.8147% -1.4284%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking initialization/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.3244 s (30k 
initialization/(Capacity: 64 Blocksize: 4096)
                        time:   [175.46 µs 176.02 µs 176.80 µs]
                        change: [+0.2608% +0.9015% +1.8927%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking initialization/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.0208 s (550k 
initialization/(Capacity: 256 Blocksize: 64)
                        time:   [9.0025 µs 9.0185 µs 9.0362 µs]
                        change: [+0.8963% +1.1516% +1.4159%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
Benchmarking initialization/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 7.2612 s (10k
initialization/(Capacity: 256 Blocksize: 4096)
                        time:   [712.29 µs 715.38 µs 719.39 µs]
                        change: [+0.7132% +1.4158% +2.3757%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

read/(Capacity: 64 Blocksize: 64)
                        time:   [284.94 ns 286.31 ns 288.70 ns]
                        change: [+0.6896% +1.0874% +1.6771%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking read/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.1773 s (116k iteration
read/(Capacity: 64 Blocksize: 4096)
                        time:   [44.902 µs 45.169 µs 45.468 µs]
                        change: [+0.7168% +1.1888% +1.6834%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking read/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.0004 s (4.7M iterations
read/(Capacity: 256 Blocksize: 64)
                        time:   [1.0597 µs 1.0664 µs 1.0757 µs]
                        change: [+0.5736% +1.7904% +3.1266%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking read/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 5.1693 s (30k iteration
read/(Capacity: 256 Blocksize: 4096)
                        time:   [170.94 µs 172.14 µs 174.11 µs]
                        change: [-0.9746% -0.0702% +0.7757%] (p = 0.88 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe

write/(Capacity: 64 Blocksize: 64)
                        time:   [267.27 ns 267.64 ns 268.05 ns]
                        change: [-23.973% -12.689% -3.7576%] (p = 0.02 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking write/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.1885 s (121k iteratio
write/(Capacity: 64 Blocksize: 4096)
                        time:   [43.061 µs 43.254 µs 43.435 µs]
                        change: [-0.9693% -0.2037% +0.4105%] (p = 0.59 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking write/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.0051 s (4.7M iteration
write/(Capacity: 256 Blocksize: 64)
                        time:   [1.0521 µs 1.0573 µs 1.0638 µs]
                        change: [-0.6369% +2.8390% +7.5006%] (p = 0.20 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking write/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 5.1740 s (30k iteratio
write/(Capacity: 256 Blocksize: 4096)
                        time:   [171.61 µs 175.83 µs 181.28 µs]
                        change: [+1.8997% +3.3885% +5.0476%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

Benchmarking random_operations/(Capacity: 64 Blocksize: 64, Ops: 64): Collecting 100 samples in estimated 5.01
random_operations/(Capacity: 64 Blocksize: 64, Ops: 64)
                        time:   [17.658 µs 17.758 µs 17.928 µs]
                        change: [-0.6032% +0.4755% +1.4591%] (p = 0.39 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking random_operations/(Capacity: 64 Blocksize: 4096, Ops: 64): Collecting 100 samples in estimated 5.
random_operations/(Capacity: 64 Blocksize: 4096, Ops: 64)
                        time:   [2.7804 ms 2.8207 ms 2.8813 ms]
                        change: [-0.3977% +1.1083% +3.4673%] (p = 0.30 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking random_operations/(Capacity: 256 Blocksize: 64, Ops: 64): Collecting 100 samples in estimated 5.1
random_operations/(Capacity: 256 Blocksize: 64, Ops: 64)
                        time:   [68.390 µs 68.491 µs 68.583 µs]
                        change: [-0.0822% +0.2091% +0.5028%] (p = 0.17 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  6 (6.00%) low mild
Benchmarking random_operations/(Capacity: 256 Blocksize: 4096, Ops: 64): Collecting 100 samples in estimated 5
random_operations/(Capacity: 256 Blocksize: 4096, Ops: 64)
                        time:   [10.822 ms 10.887 ms 10.999 ms]
                        change: [-1.2572% -0.2551% +1.0163%] (p = 0.70 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe

A logical read to ORAM with parameters: (Capacity: 64 Blocksize: 64) incurred 64 physical reads and 64 physical writes.
A logical read to ORAM with parameters: (Capacity: 64 Blocksize: 4096) incurred 64 physical reads and 64 physical writes.
A logical write to ORAM with parameters: (Capacity: 64 Blocksize: 64) incurred 64 physical reads and 64 physical writes.
A logical write to ORAM with parameters: (Capacity: 64 Blocksize: 4096) incurred 64 physical reads and 64 physical writes.
64 random ORAM operations with parameters: (Capacity: 64 Blocksize: 64, Ops: 64) incurred 4096 physical reads and 4096 physical writes.
64 random ORAM operations with parameters: (Capacity: 64 Blocksize: 4096, Ops: 64) incurred 4096 physical reads and 4096 physical writes.
A logical read to ORAM with parameters: (Capacity: 256 Blocksize: 64) incurred 256 physical reads and 256 physical writes.
A logical read to ORAM with parameters: (Capacity: 256 Blocksize: 4096) incurred 256 physical reads and 256 physical writes.
A logical write to ORAM with parameters: (Capacity: 256 Blocksize: 64) incurred 256 physical reads and 256 physical writes.
A logical write to ORAM with parameters: (Capacity: 256 Blocksize: 4096) incurred 256 physical reads and 256 physical writes.
64 random ORAM operations with parameters: (Capacity: 256 Blocksize: 64, Ops: 64) incurred 16384 physical reads and 16384 physical writes.
64 random ORAM operations with parameters: (Capacity: 256 Blocksize: 4096, Ops: 64) incurred 16384 physical reads and 16384 physical writes.
```