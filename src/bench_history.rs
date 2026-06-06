use crate::BenchResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A timestamped snapshot of benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub label: Option<String>,
    pub results: Vec<BenchResult>,
}

/// Stores historical bench results in a JSON file and tracks trends over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchHistory {
    pub entries: Vec<HistoryEntry>,
}

impl BenchHistory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Load history from a JSON file.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let history: BenchHistory = serde_json::from_str(&data)?;
        Ok(history)
    }

    /// Save history to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Add a new entry with the current timestamp.
    pub fn add(&mut self, results: Vec<BenchResult>, label: Option<String>) {
        self.entries.push(HistoryEntry {
            timestamp: Utc::now(),
            label,
            results,
        });
    }

    /// Get the latest entry.
    pub fn latest(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }

    /// Get the previous entry (second-to-last).
    pub fn previous(&self) -> Option<&HistoryEntry> {
        if self.entries.len() >= 2 {
            Some(&self.entries[self.entries.len() - 2])
        } else {
            None
        }
    }

    /// Compute the trend (linear regression slope) for a specific benchmark name.
    /// Returns the slope in ns per entry (negative = improving).
    pub fn trend(&self, benchmark_name: &str) -> Option<f64> {
        let values: Vec<f64> = self.entries.iter().filter_map(|entry| {
            entry.results.iter().find(|r| r.name == benchmark_name).map(|r| r.mean_ns)
        }).collect();

        if values.len() < 2 {
            return None;
        }

        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, v)| i as f64 * v).sum();
        let sum_xx: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator == 0.0 {
            return None;
        }

        Some((n * sum_xy - sum_x * sum_y) / denominator)
    }

    /// Get all results for a specific benchmark name across history.
    pub fn get_history_for(&self, benchmark_name: &str) -> Vec<(DateTime<Utc>, &BenchResult)> {
        self.entries.iter().filter_map(|entry| {
            entry.results.iter()
                .find(|r| r.name == benchmark_name)
                .map(|r| (entry.timestamp, r))
        }).collect()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for BenchHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, mean: f64) -> BenchResult {
        BenchResult::new(name, mean, mean, 10.0, 100)
    }

    #[test]
    fn add_and_latest() {
        let mut history = BenchHistory::new();
        history.add(vec![make_result("foo", 1000.0)], Some("v1".into()));
        history.add(vec![make_result("foo", 900.0)], Some("v2".into()));
        assert_eq!(history.len(), 2);
        assert_eq!(history.latest().unwrap().label.as_deref(), Some("v2"));
        assert_eq!(history.previous().unwrap().label.as_deref(), Some("v1"));
    }

    #[test]
    fn save_and_load() {
        let dir = std::env::temp_dir().join("bench-compare-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.json");

        let mut history = BenchHistory::new();
        history.add(vec![make_result("foo", 1000.0)], None);
        history.save(&path).unwrap();

        let loaded = BenchHistory::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.entries[0].results[0].name, "foo");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn trend_improving() {
        let mut history = BenchHistory::new();
        for i in 0..5 {
            history.add(vec![make_result("foo", 1000.0 - i as f64 * 100.0)], None);
        }
        let slope = history.trend("foo").unwrap();
        assert!(slope < 0.0, "Improving trend should have negative slope, got {}", slope);
    }

    #[test]
    fn trend_regressing() {
        let mut history = BenchHistory::new();
        for i in 0..5 {
            history.add(vec![make_result("foo", 1000.0 + i as f64 * 100.0)], None);
        }
        let slope = history.trend("foo").unwrap();
        assert!(slope > 0.0, "Regressing trend should have positive slope, got {}", slope);
    }

    #[test]
    fn trend_insufficient_data() {
        let mut history = BenchHistory::new();
        history.add(vec![make_result("foo", 1000.0)], None);
        assert!(history.trend("foo").is_none());
    }

    #[test]
    fn get_history_for_benchmark() {
        let mut history = BenchHistory::new();
        history.add(vec![make_result("foo", 1000.0), make_result("bar", 500.0)], None);
        history.add(vec![make_result("foo", 900.0), make_result("bar", 600.0)], None);

        let foo_history = history.get_history_for("foo");
        assert_eq!(foo_history.len(), 2);
        assert_eq!(foo_history[0].1.mean_ns, 1000.0);
        assert_eq!(foo_history[1].1.mean_ns, 900.0);
    }
}
