use crate::bench_comparison::{BenchComparison, ComparisonOutcome};
use crate::BenchResult;

/// Detects regressions exceeding a configurable threshold.
pub struct RegressionDetector {
    /// Percentage threshold above which a change is flagged as a regression.
    pub threshold_percent: f64,
}

impl RegressionDetector {
    pub fn new(threshold_percent: f64) -> Self {
        Self { threshold_percent }
    }

    /// Compare two result sets and return only regressions.
    pub fn detect(&self, baseline: &[BenchResult], current: &[BenchResult]) -> Vec<BenchComparison> {
        let all = BenchComparison::compare_sets(baseline, current, self.threshold_percent);
        all.into_iter()
            .filter(|c| matches!(c.outcome, ComparisonOutcome::Regressed { .. }))
            .collect()
    }

    /// Check if any regression was detected.
    pub fn has_regressions(&self, baseline: &[BenchResult], current: &[BenchResult]) -> bool {
        !self.detect(baseline, current).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, mean: f64) -> BenchResult {
        BenchResult::new(name, mean, mean, 10.0, 100)
    }

    #[test]
    fn detects_regression() {
        let detector = RegressionDetector::new(10.0);
        let baseline = vec![make_result("foo", 1000.0)];
        let current = vec![make_result("foo", 1200.0)]; // +20%
        let regressions = detector.detect(&baseline, &current);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].name, "foo");
    }

    #[test]
    fn no_regression_when_improved() {
        let detector = RegressionDetector::new(10.0);
        let baseline = vec![make_result("foo", 1000.0)];
        let current = vec![make_result("foo", 800.0)]; // -20%
        assert!(!detector.has_regressions(&baseline, &current));
    }

    #[test]
    fn no_regression_within_threshold() {
        let detector = RegressionDetector::new(10.0);
        let baseline = vec![make_result("foo", 1000.0)];
        let current = vec![make_result("foo", 1050.0)]; // +5%
        assert!(!detector.has_regressions(&baseline, &current));
    }

    #[test]
    fn multiple_regressions() {
        let detector = RegressionDetector::new(5.0);
        let baseline = vec![
            make_result("a", 1000.0),
            make_result("b", 2000.0),
            make_result("c", 3000.0),
        ];
        let current = vec![
            make_result("a", 1100.0), // +10%
            make_result("b", 2200.0), // +10%
            make_result("c", 2800.0), // improved
        ];
        let regressions = detector.detect(&baseline, &current);
        assert_eq!(regressions.len(), 2);
    }
}
