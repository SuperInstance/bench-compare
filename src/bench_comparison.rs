use crate::BenchResult;

/// Outcome of comparing a single benchmark between two runs.
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOutcome {
    /// The benchmark improved (got faster).
    Improved { percent: f64 },
    /// The benchmark is essentially unchanged.
    Unchanged { percent: f64 },
    /// The benchmark regressed (got slower).
    Regressed { percent: f64 },
    /// Benchmark exists in only one run.
    OnlyInOne { which: String },
}

/// Comparison of a single benchmark between baseline and current.
#[derive(Debug, Clone)]
pub struct BenchComparison {
    /// Benchmark name
    pub name: String,
    /// Baseline (old) result
    pub baseline: Option<BenchResult>,
    /// Current (new) result
    pub current: Option<BenchResult>,
    /// Percentage change: positive = slower, negative = faster
    pub change_percent: f64,
    /// Confidence interval half-width in percent
    pub confidence_interval: f64,
    /// The outcome
    pub outcome: ComparisonOutcome,
}

impl BenchComparison {
    /// Compare two BenchResults by mean time.
    /// `threshold_percent` is the boundary for "unchanged" (e.g. 5.0 means ±5% is unchanged).
    pub fn compare(baseline: &BenchResult, current: &BenchResult, threshold_percent: f64) -> Self {
        let change_percent = ((current.mean_ns - baseline.mean_ns) / baseline.mean_ns) * 100.0;
        // Simple CI: combine relative errors in quadrature
        let baseline_rel = baseline.relative_error();
        let current_rel = current.relative_error();
        let combined_rel = (baseline_rel.powi(2) + current_rel.powi(2)).sqrt();
        let confidence_interval = combined_rel * 100.0;

        let outcome = if change_percent < -threshold_percent {
            ComparisonOutcome::Improved { percent: change_percent }
        } else if change_percent > threshold_percent {
            ComparisonOutcome::Regressed { percent: change_percent }
        } else {
            ComparisonOutcome::Unchanged { percent: change_percent }
        };

        Self {
            name: baseline.name.clone(),
            baseline: Some(baseline.clone()),
            current: Some(current.clone()),
            change_percent,
            confidence_interval,
            outcome,
        }
    }

    /// Compare two full sets of benchmark results, matching by name.
    pub fn compare_sets(
        baseline: &[BenchResult],
        current: &[BenchResult],
        threshold_percent: f64,
    ) -> Vec<Self> {
        let mut comparisons = Vec::new();
        let baseline_map: std::collections::HashMap<&str, &BenchResult> = baseline
            .iter()
            .map(|b| (b.name.as_str(), b))
            .collect();
        let current_map: std::collections::HashMap<&str, &BenchResult> = current
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();

        // All names union
        let mut all_names: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for name in baseline_map.keys() {
            all_names.insert(name);
        }
        for name in current_map.keys() {
            all_names.insert(name);
        }

        for name in all_names {
            let b = baseline_map.get(name);
            let c = current_map.get(name);

            match (b, c) {
                (Some(b), Some(c)) => {
                    comparisons.push(Self::compare(b, c, threshold_percent));
                }
                (Some(b), None) => {
                    comparisons.push(Self {
                        name: b.name.clone(),
                        baseline: Some((*b).clone()),
                        current: None,
                        change_percent: f64::NAN,
                        confidence_interval: f64::NAN,
                        outcome: ComparisonOutcome::OnlyInOne { which: "baseline".into() },
                    });
                }
                (None, Some(c)) => {
                    comparisons.push(Self {
                        name: c.name.clone(),
                        baseline: None,
                        current: Some((*c).clone()),
                        change_percent: f64::NAN,
                        confidence_interval: f64::NAN,
                        outcome: ComparisonOutcome::OnlyInOne { which: "current".into() },
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        comparisons
    }

    /// Speedup ratio (>1 means current is faster).
    pub fn speedup(&self) -> Option<f64> {
        match (&self.baseline, &self.current) {
            (Some(b), Some(c)) if b.mean_ns > 0.0 => Some(b.mean_ns / c.mean_ns),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, mean: f64, stddev: f64) -> BenchResult {
        BenchResult::new(name, mean, mean, stddev, 100)
    }

    #[test]
    fn compare_improved() {
        let b = make_result("foo", 1000.0, 10.0);
        let c = make_result("foo", 800.0, 10.0);
        let comp = BenchComparison::compare(&b, &c, 5.0);
        assert!(matches!(comp.outcome, ComparisonOutcome::Improved { .. }));
        assert_eq!(comp.change_percent, -20.0);
        assert!(comp.speedup().unwrap() > 1.0);
    }

    #[test]
    fn compare_regressed() {
        let b = make_result("foo", 1000.0, 10.0);
        let c = make_result("foo", 1200.0, 10.0);
        let comp = BenchComparison::compare(&b, &c, 5.0);
        assert!(matches!(comp.outcome, ComparisonOutcome::Regressed { .. }));
        assert_eq!(comp.change_percent, 20.0);
    }

    #[test]
    fn compare_unchanged() {
        let b = make_result("foo", 1000.0, 10.0);
        let c = make_result("foo", 1020.0, 10.0);
        let comp = BenchComparison::compare(&b, &c, 5.0);
        assert!(matches!(comp.outcome, ComparisonOutcome::Unchanged { .. }));
    }

    #[test]
    fn compare_sets_mixed() {
        let baseline = vec![
            make_result("a", 1000.0, 10.0),
            make_result("b", 2000.0, 20.0),
            make_result("c", 3000.0, 30.0),
        ];
        let current = vec![
            make_result("a", 900.0, 10.0),  // improved
            make_result("b", 2500.0, 20.0), // regressed
            make_result("d", 500.0, 5.0),   // new
        ];

        let comps = BenchComparison::compare_sets(&baseline, &current, 5.0);
        assert_eq!(comps.len(), 4);

        let a = comps.iter().find(|c| c.name == "a").unwrap();
        assert!(matches!(a.outcome, ComparisonOutcome::Improved { .. }));

        let b_comp = comps.iter().find(|c| c.name == "b").unwrap();
        assert!(matches!(b_comp.outcome, ComparisonOutcome::Regressed { .. }));

        let c_comp = comps.iter().find(|c| c.name == "c").unwrap();
        assert!(matches!(c_comp.outcome, ComparisonOutcome::OnlyInOne { .. }));

        let d = comps.iter().find(|c| c.name == "d").unwrap();
        assert!(matches!(d.outcome, ComparisonOutcome::OnlyInOne { .. }));
    }
}
