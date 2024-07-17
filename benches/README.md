# Running benchmarks

Use `cargo bench` to run benchmarks.

# Example benchmark output

```
% cargo bench
Running benches/benchmark.rs (target/release/deps/benchmark-c0e9411356248ba6)
LinearTimeOram::initialization/(Capacity: 64 Blocksize: 64)
                        time:   [66.619 ns 67.973 ns 69.590 ns]
                        change: [+0.2232% +4.1831% +8.2459%] (p = 0.04 < 0.05)
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
LinearTimeOram::initialization/(Capacity: 256 Blocksize: 64)
                        time:   [212.72 ns 223.59 ns 235.90 ns]
                        change: [-11.858% -6.0727% +1.6889%] (p = 0.08 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

LinearTimeOram::initialization/(Capacity: 64 Blocksize: 4096)
                        time:   [3.6845 µs 3.9785 µs 4.2586 µs]
                        change: [+4.4178% +13.170% +23.508%] (p = 0.00 < 0.05)
                        Performance has regressed.
LinearTimeOram::initialization/(Capacity: 256 Blocksize: 4096)
                        time:   [12.311 µs 13.262 µs 14.343 µs]
                        change: [-12.499% -4.1684% +4.2933%] (p = 0.32 > 0.05)
                        No change in performance detected.

LinearTimeOram::read/(Capacity: 64 Blocksize: 64)
                        time:   [293.91 ns 296.46 ns 301.13 ns]
                        change: [+3.7833% +4.6460% +5.9849%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
LinearTimeOram::read/(Capacity: 256 Blocksize: 64)
                        time:   [1.0843 µs 1.0916 µs 1.1028 µs]
                        change: [+1.8924% +4.2521% +7.6769%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low mild
  13 (13.00%) high mild
  2 (2.00%) high severe

LinearTimeOram::read/(Capacity: 64 Blocksize: 4096)
                        time:   [43.301 µs 43.401 µs 43.526 µs]
                        change: [-7.0116% -1.8092% +1.7788%] (p = 0.61 > 0.05)
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
LinearTimeOram::read/(Capacity: 256 Blocksize: 4096)
                        time:   [169.20 µs 169.80 µs 170.56 µs]
                        change: [+1.6632% +2.1063% +2.5624%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe

LinearTimeOram::write/(Capacity: 64 Blocksize: 64)
                        time:   [275.86 ns 276.25 ns 276.70 ns]
                        change: [+1.5245% +2.0471% +2.4350%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 23 outliers among 100 measurements (23.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  6 (6.00%) high mild
  11 (11.00%) high severe
LinearTimeOram::write/(Capacity: 256 Blocksize: 64)
                        time:   [1.0674 µs 1.0688 µs 1.0706 µs]
                        change: [-1.8993% -0.3239% +0.6033%] (p = 0.77 > 0.05)
                        No change in performance detected.
Found 23 outliers among 100 measurements (23.00%)
  4 (4.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  12 (12.00%) high severe

LinearTimeOram::write/(Capacity: 64 Blocksize: 4096)
                        time:   [42.755 µs 42.809 µs 42.869 µs]
                        change: [-0.9097% +0.7446% +1.8029%] (p = 0.38 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
LinearTimeOram::write/(Capacity: 256 Blocksize: 4096)
                        time:   [170.36 µs 174.95 µs 180.99 µs]
                        change: [+1.4177% +3.5344% +6.0682%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  9 (9.00%) high severe

LinearTimeOram::random_operations/(Capacity: 64 Blocksize: 64, Ops: 64)
                        time:   [18.025 µs 18.050 µs 18.081 µs]
                        change: [-0.2062% +0.6009% +1.2096%] (p = 0.10 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
LinearTimeOram::random_operations/(Capacity: 256 Blocksize: 64, Ops: 64)
                        time:   [68.797 µs 68.942 µs 69.102 µs]
                        change: [+0.7674% +1.0205% +1.2420%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  2 (2.00%) high mild
  5 (5.00%) high severe

LinearTimeOram::random_operations/(Capacity: 64 Blocksize: 4096, Ops: 64)
                        time:   [2.7431 ms 2.7468 ms 2.7507 ms]
                        change: [+0.1405% +1.2075% +1.8350%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
LinearTimeOram::random_operations/(Capacity: 256 Blocksize: 4096, Ops: 64)
                        time:   [10.842 ms 10.853 ms 10.865 ms]
                        change: [-0.5816% -0.0661% +0.3168%] (p = 0.81 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Physical reads and writes incurred by 1 LinearTimeOram::read:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 64              | 64             
256             | 64              | 256             | 256            
64              | 4096            | 64              | 64             
256             | 4096            | 256             | 256            

Physical reads and writes incurred by 1 LinearTimeOram::write:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 64              | 64             
256             | 64              | 256             | 256            
64              | 4096            | 64              | 64             
256             | 4096            | 256             | 256            

Physical reads and writes incurred by 64 random LinearTimeOram operations:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 4096            | 4096           
256             | 64              | 16384           | 16384          
64              | 4096            | 4096            | 4096           
256             | 4096            | 16384           | 16384          
VecPathOram::initialization/(Capacity: 64 Blocksize: 64)
                        time:   [14.776 µs 15.115 µs 15.423 µs]
                        change: [-5.9812% -3.1459% -0.5428%] (p = 0.03 < 0.05)
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
VecPathOram::initialization/(Capacity: 256 Blocksize: 64)
                        time:   [77.748 µs 80.536 µs 83.473 µs]
                        change: [-2.6343% +2.0805% +6.8860%] (p = 0.39 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

VecPathOram::initialization/(Capacity: 64 Blocksize: 4096)
                        time:   [101.55 µs 104.35 µs 108.57 µs]
                        change: [+0.7226% +3.9578% +8.0572%] (p = 0.02 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
VecPathOram::initialization/(Capacity: 256 Blocksize: 4096)
                        time:   [478.85 µs 490.71 µs 501.74 µs]
                        change: [+7.6267% +12.171% +16.204%] (p = 0.00 < 0.05)
                        Performance has regressed.

VecPathOram::read/(Capacity: 64 Blocksize: 64)
                        time:   [20.581 µs 20.629 µs 20.685 µs]
                        change: [-3.2297% -0.3683% +1.4998%] (p = 0.83 > 0.05)
                        No change in performance detected.
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  16 (16.00%) high severe
VecPathOram::read/(Capacity: 256 Blocksize: 64)
                        time:   [87.103 µs 87.290 µs 87.504 µs]
                        change: [+1.4341% +1.8262% +2.2138%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  8 (8.00%) high severe

Benchmarking VecPathOram::read/(Capacity: 64 Blocksize: 4096): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.7s, enable flat sampling, or reduce sample count to 60.
VecPathOram::read/(Capacity: 64 Blocksize: 4096)
                        time:   [1.1239 ms 1.1264 ms 1.1294 ms]
                        change: [-11.206% -4.5354% -0.1075%] (p = 0.16 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) high mild
  3 (3.00%) high severe
VecPathOram::read/(Capacity: 256 Blocksize: 4096)
                        time:   [4.8769 ms 4.8924 ms 4.9129 ms]
                        change: [-0.1595% +0.4406% +1.0812%] (p = 0.16 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

VecPathOram::write/(Capacity: 64 Blocksize: 64)
                        time:   [20.633 µs 20.669 µs 20.706 µs]
                        change: [+1.4562% +1.7694% +2.0853%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  5 (5.00%) high mild
  1 (1.00%) high severe
VecPathOram::write/(Capacity: 256 Blocksize: 64)
                        time:   [87.523 µs 87.675 µs 87.833 µs]
                        change: [+0.6092% +0.9128% +1.2965%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking VecPathOram::write/(Capacity: 64 Blocksize: 4096): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.0s, enable flat sampling, or reduce sample count to 60.
VecPathOram::write/(Capacity: 64 Blocksize: 4096)
                        time:   [1.1261 ms 1.1278 ms 1.1298 ms]
                        change: [-1.8599% -0.2878% +0.8158%] (p = 0.75 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  2 (2.00%) high severe
VecPathOram::write/(Capacity: 256 Blocksize: 4096)
                        time:   [4.8705 ms 4.8761 ms 4.8823 ms]
                        change: [+0.2058% +0.8607% +1.3014%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking VecPathOram::random_operations/(Capacity: 64 Blocksize: 64, Ops: 64): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.7s, enable flat sampling, or reduce sample count to 60.
VecPathOram::random_operations/(Capacity: 64 Blocksize: 64, Ops: 64)
                        time:   [1.3177 ms 1.3201 ms 1.3231 ms]
                        change: [+0.7395% +1.0967% +1.4394%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
VecPathOram::random_operations/(Capacity: 256 Blocksize: 64, Ops: 64)
                        time:   [3.6720 ms 3.6826 ms 3.6991 ms]
                        change: [-3.2277% -0.5866% +0.9930%] (p = 0.77 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

Benchmarking VecPathOram::random_operations/(Capacity: 64 Blocksize: 4096, Ops: 64): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.2s, or reduce sample count to 60.
VecPathOram::random_operations/(Capacity: 64 Blocksize: 4096, Ops: 64)
                        time:   [72.102 ms 72.292 ms 72.605 ms]
                        change: [+1.3935% +1.7063% +2.1581%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking VecPathOram::random_operations/(Capacity: 256 Blocksize: 4096, Ops: 64): Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 20.1s, or reduce sample count to 20.
VecPathOram::random_operations/(Capacity: 256 Blocksize: 4096, Ops: 64)
                        time:   [209.62 ms 209.81 ms 210.05 ms]
                        change: [+0.8107% +1.0270% +1.2406%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Physical reads and writes incurred by 1 VecPathOram::read:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 6               | 6              
256             | 64              | 8               | 8              
64              | 4096            | 6               | 6              
256             | 4096            | 8               | 8              

Physical reads and writes incurred by 1 VecPathOram::write:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 6               | 6              
256             | 64              | 8               | 8              
64              | 4096            | 6               | 6              
256             | 4096            | 8               | 8              

Physical reads and writes incurred by 64 random VecPathOram operations:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 384             | 384            
256             | 64              | 512             | 512            
64              | 4096            | 384             | 384            
256             | 4096            | 512             | 512     
```