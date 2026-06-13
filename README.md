# Bench Compare

**Bench Compare** is a Rust library for parsing, comparing, and tracking `cargo bench` results with regression detection — it extracts benchmark metrics from Criterion output, computes statistical significance via Welch's t-test, and maintains historical baselines for performance monitoring.

## Why It Matters

Performance regressions are the most insidious bugs: no test fails, no assertion trips, but the system silently slows down. Without continuous benchmark tracking, regressions from dependency updates, compiler changes, or subtle algorithmic modifications go undetected for weeks. Bench Compare automates the regression detection pipeline: parse Criterion's JSON output, compare against stored baselines, compute whether the delta is statistically significant (not just noise), and alert when performance degrades beyond a threshold. This is the same capability provided by `cargo bench` + Criterion's built-in comparison, but as a standalone library that can integrate with any CI pipeline, custom benchmarking harness, or fleet monitoring system.

## How It Works

**Parsing:** The `BenchParser` extracts structured `BenchResult` records from Criterion output:

```
BenchResult {
    name: "bench_vector_push",
    mean_ns: 142.3,
    median_ns: 138.7,
    stddev_ns: 12.1,
    iterations: 100000,
}
```

**Comparison:** `BenchComparison` pairs baseline vs. current results by name. The outcome is classified:

```
ComparisonOutcome:
  Improvement (current < baseline by > threshold)
  Regression  (current > baseline by > threshold)
  NoChange    (|delta| ≤ threshold)
```

The threshold is configurable as either a percentage (e.g., ±5%) or as a multiple of the standard deviation (e.g., ±2σ).

**Statistical significance:** `RegressionDetector` applies a Welch's t-test to the raw sample distributions:

```
t = (mean_a − mean_b) / √(var_a/n_a + var_b/n_b)
```

With Welch-Satterthwaite degrees of freedom approximation. A regression is flagged only when both: (1) the effect size exceeds the threshold, AND (2) the t-test reaches significance at the configured α level (default 0.05). This dual criterion prevents both false positives (statistically significant but practically negligible changes) and false negatives (large changes masked by high variance).

**History tracking:** `BenchHistory` maintains a time-series of benchmark results, enabling trend analysis. Stored as JSON for portability:

```json
{"bench_name": [{"date": "2026-01-15", "mean_ns": 142.3}, ...]}
```

## Quick Start

```rust
use bench_compare::{BenchResult, BenchComparison, ComparisonOutcome};

fn main() {
    let baseline = BenchResult::new("hash_bench", 1000.0, 990.0, 50.0, 10000);
    let current = BenchResult::new("hash_bench", 1150.0, 1140.0, 45.0, 10000);

    let comparison = BenchComparison::new(&baseline, &current);
    println!("Delta: {:.1}%", comparison.delta_pct());
    // Regression if > 5% slower
}
```

## API

| Type | Description |
|------|-------------|
| `BenchResult` | Single benchmark: name, mean, median, stddev, iterations |
| `BenchParser` | Extract results from Criterion output |
| `BenchComparison` | Baseline vs. current comparison |
| `ComparisonOutcome` | Improvement, Regression, or NoChange |
| `RegressionDetector` | Statistical significance testing |
| `BenchHistory` | Time-series baseline storage |
| `BenchReport` | Formatted output generation |

## Architecture Notes

Bench Compare provides the **performance regression monitoring** for the SuperInstance fleet. Within γ + η = C, it tracks whether conservation-law computations maintain their performance baseline as the fleet grows — ensuring that conservation verification (a timing-sensitive real-time process) does not regress as new agents and services are added.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Welch, B.L. (1947). "The Generalization of 'Student's' Problem When Several Different Population Variances Are Involved." *Biometrika*, 34(1/2), 28–35.
2. Kalibera, T. & Jones, R. (2013). "Rigorous Benchmarking in Reasonable Time." *ISSTA*.

## License

MIT
