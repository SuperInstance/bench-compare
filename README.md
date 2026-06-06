# bench-compare

Parse, compare, and track `cargo bench` results with regression detection.

## Features

- **BenchParser** — Parse standard `cargo bench` output into structured `BenchResult` structs
- **BenchComparison** — Compare two benchmark runs, compute speedup/slowdown with confidence intervals
- **RegressionDetector** — Flag benchmarks that regressed beyond a configurable threshold
- **BenchHistory** — Store historical results in JSON, compute trends over time via linear regression
- **BenchReport** — Generate markdown reports with 🟢 improved / 🟡 unchanged / 🔴 regressed indicators

## Usage

```rust
use bench_compare::{BenchParser, BenchComparison, RegressionDetector, BenchHistory, BenchReport};

// Parse cargo bench output
let baseline = BenchParser::parse(&baseline_output);
let current = BenchParser::parse(&current_output);

// Compare
let comparisons = BenchComparison::compare_sets(&baseline, &current, 5.0);

// Detect regressions
let detector = RegressionDetector::new(10.0);
let regressions = detector.detect(&baseline, &current);

// Generate report
let report = BenchReport::generate(&comparisons);
println!("{}", report);

// Track over time
let mut history = BenchHistory::new();
history.add(current, Some("v2.0".into()));
history.save(std::path::Path::new("bench-history.json")).unwrap();
```

## Benchmark Output Format

Supports standard libtest benchmark output:

```
test bench_foo ... bench:       2,345 ns/iter (+/- 120)
```
