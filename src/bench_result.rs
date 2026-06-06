use serde::{Deserialize, Serialize};

/// A single benchmark result parsed from `cargo bench` output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchResult {
    /// Benchmark function/test name
    pub name: String,
    /// Mean execution time in nanoseconds
    pub mean_ns: f64,
    /// Median execution time in nanoseconds
    pub median_ns: f64,
    /// Standard deviation in nanoseconds
    pub stddev_ns: f64,
    /// Number of iterations performed
    pub iterations: u64,
}

impl BenchResult {
    pub fn new(name: impl Into<String>, mean_ns: f64, median_ns: f64, stddev_ns: f64, iterations: u64) -> Self {
        Self {
            name: name.into(),
            mean_ns,
            median_ns,
            stddev_ns,
            iterations,
        }
    }

    /// Convert nanoseconds to microseconds.
    pub fn mean_us(&self) -> f64 {
        self.mean_ns / 1_000.0
    }

    /// Convert nanoseconds to milliseconds.
    pub fn mean_ms(&self) -> f64 {
        self.mean_ns / 1_000_000.0
    }

    /// Relative margin of error (stddev / mean).
    pub fn relative_error(&self) -> f64 {
        if self.mean_ns == 0.0 {
            0.0
        } else {
            self.stddev_ns / self.mean_ns
        }
    }
}
