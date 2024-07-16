# Running benchmarks

Use `cargo bench` to run benchmarks.

# Example benchmark output

```
% cargo bench
Running benches/benchmark.rs (target/release/deps/benchmark-0372a3041da7f352)
LinearTimeOram::read/(Capacity: 16384 Blocksize: 4096)
                        time:   [9.1764 ms 9.2246 ms 9.2783 ms]
                        change: [-12.509% -11.377% -10.272%] (p = 0.00 < 0.05)
                        Performance has improved.
Benchmarking LinearTimeOram::read/(Capacity: 65536 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 407.7ms.
LinearTimeOram::read/(Capacity: 65536 Blocksize: 4096)
                        time:   [36.470 ms 36.788 ms 37.225 ms]
                        change: [-14.398% -12.615% -10.898%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
Benchmarking LinearTimeOram::read/(Capacity: 1048576 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 17.5s.
LinearTimeOram::read/(Capacity: 1048576 Blocksize: 4096)
                        time:   [601.12 ms 662.39 ms 762.15 ms]
                        change: [-10.865% -1.3507% +15.145%] (p = 0.88 > 0.05)
                        No change in performance detected.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe

RecursiveSecureOram::read/(Capacity: 16384 Blocksize: 4096)
                        time:   [1.1581 ms 1.1600 ms 1.1620 ms]
                        change: [-12.640% -11.624% -10.601%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild
RecursiveSecureOram::read/(Capacity: 65536 Blocksize: 4096)
                        time:   [1.3746 ms 1.4106 ms 1.4715 ms]
                        change: [-9.6176% -7.3772% -4.2653%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
RecursiveSecureOram::read/(Capacity: 1048576 Blocksize: 4096)
                        time:   [4.9352 ms 4.9622 ms 4.9939 ms]
                        change: [-21.182% -16.699% -12.677%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild

LinearTimeOram::initialization/(Capacity: 16384 Blocksize: 4096)
                        time:   [8.0183 ms 8.2087 ms 8.3918 ms]
                        change: [+8.6264% +12.309% +16.306%] (p = 0.00 < 0.05)
                        Performance has regressed.
Benchmarking LinearTimeOram::initialization/(Capacity: 65536 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 310.0ms.
LinearTimeOram::initialization/(Capacity: 65536 Blocksize: 4096)
                        time:   [32.430 ms 33.623 ms 35.165 ms]
                        change: [-7.8815% -4.1354% +0.2933%] (p = 0.09 > 0.05)
                        No change in performance detected.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild
Benchmarking LinearTimeOram::initialization/(Capacity: 1048576 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 4.2s.
LinearTimeOram::initialization/(Capacity: 1048576 Blocksize: 4096)
                        time:   [417.29 ms 422.91 ms 429.42 ms]
                        change: [-8.8642% -5.6243% -2.5261%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 10 measurements (30.00%)
  1 (10.00%) low mild
  2 (20.00%) high severe

Benchmarking RecursiveSecureOram::initialization/(Capacity: 16384 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 490.9ms.
RecursiveSecureOram::initialization/(Capacity: 16384 Blocksize: 4096)
                        time:   [50.339 ms 50.929 ms 51.417 ms]
                        change: [-20.147% -11.930% -6.2350%] (p = 0.01 < 0.05)
                        Performance has improved.
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) low mild
Benchmarking RecursiveSecureOram::initialization/(Capacity: 65536 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 2.0s.
RecursiveSecureOram::initialization/(Capacity: 65536 Blocksize: 4096)
                        time:   [194.39 ms 194.83 ms 195.19 ms]
                        change: [-2.4988% -1.7890% -1.0964%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 10 measurements (30.00%)
  1 (10.00%) low severe
  2 (20.00%) high mild
Benchmarking RecursiveSecureOram::initialization/(Capacity: 1048576 Blocksize: 4096): Warming up for 100.00 ms
Warning: Unable to complete 10 samples in 100.0ms. You may wish to increase target time to 63.6s.
RecursiveSecureOram::initialization/(Capacity: 1048576 Blocksize: 4096)
                        time:   [6.3310 s 6.4535 s 6.6195 s]
                        change: [-7.1614% -4.4196% -1.2474%] (p = 0.02 < 0.05)
                        Performance has improved.
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe

Physical reads and writes incurred by 1 LinearTimeOram::read:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
16384           | 4096            | 16384           | 16384          
65536           | 4096            | 65536           | 65536          
1048576         | 4096            | 1048576         | 1048576        
Physical reads and writes incurred by 1 RecursiveSecureOram::read:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
16384           | 4096            | 14              | 14             
65536           | 4096            | 16              | 16             
1048576         | 4096            | 20              | 20     
```