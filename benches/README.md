# Running benchmarks

Use `cargo bench` to run benchmarks.

# Example benchmark output

```
% cargo bench
Benchmarking initialization/(Capacity: 64 Blocksize: 64): Collecting 100 samples in estimated 5.000
initialization/(Capacity: 64 Blocksize: 64)
                        time:   [66.170 ns 67.941 ns 69.942 ns]
                        change: [-10.227% -5.9626% -1.2930%] (p = 0.01 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking initialization/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.00
initialization/(Capacity: 256 Blocksize: 64)
                        time:   [205.15 ns 213.96 ns 224.21 ns]
                        change: [-3.1070% +2.3692% +7.9817%] (p = 0.42 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking initialization/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.0
initialization/(Capacity: 64 Blocksize: 4096)
                        time:   [4.4354 µs 4.4364 µs 4.4375 µs]
                        change: [-1.1342% -0.7390% -0.4277%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking initialization/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 5.
initialization/(Capacity: 256 Blocksize: 4096)
                        time:   [18.477 µs 18.942 µs 19.640 µs]
                        change: [-0.3358% +1.5578% +3.5248%] (p = 0.19 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking read/(Capacity: 64 Blocksize: 64): Collecting 100 samples in estimated 5.0005 s (18M i
read/(Capacity: 64 Blocksize: 64)
                        time:   [274.22 ns 274.61 ns 275.13 ns]
                        change: [-3.9419% -2.2790% -1.0297%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) high mild
  8 (8.00%) high severe
Benchmarking read/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.0037 s (4.9M
read/(Capacity: 256 Blocksize: 64)
                        time:   [1.0233 µs 1.0253 µs 1.0276 µs]
                        change: [-1.6698% -1.3647% -1.0760%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking read/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.1083 s (121
read/(Capacity: 64 Blocksize: 4096)
                        time:   [41.871 µs 41.973 µs 42.100 µs]
                        change: [-0.8742% -0.4453% +0.0112%] (p = 0.04 < 0.05)
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking read/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 5.8015 s (35
read/(Capacity: 256 Blocksize: 4096)
                        time:   [164.04 µs 168.15 µs 173.83 µs]
                        change: [-1.9313% -0.8973% +0.6455%] (p = 0.16 > 0.05)
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking write/(Capacity: 64 Blocksize: 64): Collecting 100 samples in estimated 5.0000 s (19M 
write/(Capacity: 64 Blocksize: 64)
                        time:   [260.26 ns 261.08 ns 262.32 ns]
                        change: [+0.4073% +0.7040% +1.0049%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking write/(Capacity: 256 Blocksize: 64): Collecting 100 samples in estimated 5.0048 s (4.9
write/(Capacity: 256 Blocksize: 64)
                        time:   [1.0061 µs 1.0073 µs 1.0088 µs]
                        change: [-1.7819% -1.4074% -1.0276%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking write/(Capacity: 64 Blocksize: 4096): Collecting 100 samples in estimated 5.1228 s (12
write/(Capacity: 64 Blocksize: 4096)
                        time:   [41.865 µs 42.015 µs 42.180 µs]
                        change: [-1.1280% -0.6789% -0.2446%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking write/(Capacity: 256 Blocksize: 4096): Collecting 100 samples in estimated 5.8303 s (3
write/(Capacity: 256 Blocksize: 4096)
                        time:   [164.14 µs 169.96 µs 177.88 µs]
                        change: [-3.0423% -0.9423% +2.1075%] (p = 0.58 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking random_operations/(Capacity: 64 Blocksize: 64, Ops: 64): Collecting 100 samples in est
random_operations/(Capacity: 64 Blocksize: 64, Ops: 64)
                        time:   [17.167 µs 17.367 µs 17.738 µs]
                        change: [-0.4767% +2.5944% +7.6874%] (p = 0.35 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking random_operations/(Capacity: 256 Blocksize: 64, Ops: 64): Collecting 100 samples in es
random_operations/(Capacity: 256 Blocksize: 64, Ops: 64)
                        time:   [65.393 µs 65.694 µs 66.070 µs]
                        change: [-1.0442% -0.3388% +0.3291%] (p = 0.34 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

Benchmarking random_operations/(Capacity: 64 Blocksize: 64, Ops: 64) #2: Collecting 100 samples in 
random_operations/(Capacity: 64 Blocksize: 64, Ops: 64) #2
                        time:   [17.436 µs 17.862 µs 18.352 µs]
Found 17 outliers among 100 measurements (17.00%)
  4 (4.00%) low mild
  7 (7.00%) high mild
  6 (6.00%) high severe
random_operations/(Capacity: 256 Blocksize: 64, Ops: 64) #2
                        time:   [65.318 µs 65.438 µs 65.567 µs]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild

Physical reads and writes incurred by 1 ORAM read:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 64              | 64             
256             | 64              | 256             | 256            
64              | 4096            | 64              | 64             
256             | 4096            | 256             | 256            

Physical reads and writes incurred by 1 ORAM write:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 64              | 64             
256             | 64              | 256             | 256            
64              | 4096            | 64              | 64             
256             | 4096            | 256             | 256        

Physical reads and writes incurred by 64 random ORAM operations:
ORAM Capacity   | ORAM Blocksize  | Physical Reads  | Physical Writes
64              | 64              | 4096            | 4096           
256             | 64              | 16384           | 16384          
64              | 4096            | 4096            | 4096           
256             | 4096            | 16384           | 16384          
```