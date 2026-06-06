use crate::bench_comparison::{BenchComparison, ComparisonOutcome};

/// Generates markdown reports from benchmark comparisons.
pub struct BenchReport;

impl BenchReport {
    /// Generate a markdown report from a list of comparisons.
    pub fn generate(comparisons: &[BenchComparison]) -> String {
        let mut md = String::new();
        md.push_str("# Benchmark Report\n\n");

        let improved = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::Improved { .. })).count();
        let regressed = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::Regressed { .. })).count();
        let unchanged = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::Unchanged { .. })).count();
        let only_one = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::OnlyInOne { .. })).count();

        md.push_str(&format!("**Summary:** {} 🟢 improved | {} 🟡 unchanged | {} 🔴 regressed", improved, unchanged, regressed));
        if only_one > 0 {
            md.push_str(&format!(" | {} ➖ only in one run", only_one));
        }
        md.push_str("\n\n");

        md.push_str("| Benchmark | Baseline | Current | Change | Status |\n");
        md.push_str("|-----------|----------|---------|--------|--------|\n");

        for comp in comparisons {
            let baseline_str = comp.baseline.as_ref()
                .map(|b| format!("{:.0} ns", b.mean_ns))
                .unwrap_or_else(|| "—".into());
            let current_str = comp.current.as_ref()
                .map(|c| format!("{:.0} ns", c.mean_ns))
                .unwrap_or_else(|| "—".into());

            let (change_str, indicator) = match &comp.outcome {
                ComparisonOutcome::Improved { percent } => (format!("{:+.1}%", percent), "🟢"),
                ComparisonOutcome::Unchanged { percent } => (format!("{:+.1}%", percent), "🟡"),
                ComparisonOutcome::Regressed { percent } => (format!("{:+.1}%", percent), "🔴"),
                ComparisonOutcome::OnlyInOne { which } => (format!("only in {}", which), "➖"),
            };

            md.push_str(&format!("| {} | {} | {} | {} | {} |\n",
                comp.name, baseline_str, current_str, change_str, indicator));
        }

        md
    }

    /// Generate a brief summary string (suitable for CI comments).
    pub fn summary(comparisons: &[BenchComparison]) -> String {
        let regressed = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::Regressed { .. })).count();
        let improved = comparisons.iter().filter(|c| matches!(c.outcome, ComparisonOutcome::Improved { .. })).count();
        let total = comparisons.len();

        if regressed == 0 && improved > 0 {
            format!("✅ No regressions across {} benchmarks ({} improved)", total, improved)
        } else if regressed > 0 {
            format!("⚠️ {} regression(s) detected out of {} benchmarks", regressed, total)
        } else {
            format!("✅ All {} benchmarks unchanged", total)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchResult;

    fn make_result(name: &str, mean: f64) -> BenchResult {
        BenchResult::new(name, mean, mean, 10.0, 100)
    }

    #[test]
    fn report_contains_all_sections() {
        let baseline = vec![make_result("a", 1000.0), make_result("b", 2000.0)];
        let current = vec![make_result("a", 800.0), make_result("b", 2500.0)];
        let comps = crate::bench_comparison::BenchComparison::compare_sets(&baseline, &current, 5.0);
        let report = BenchReport::generate(&comps);

        assert!(report.contains("# Benchmark Report"));
        assert!(report.contains("🟢"));
        assert!(report.contains("🔴"));
        assert!(report.contains("| a |"));
    }

    #[test]
    fn summary_no_regressions() {
        let baseline = vec![make_result("a", 1000.0)];
        let current = vec![make_result("a", 900.0)];
        let comps = crate::bench_comparison::BenchComparison::compare_sets(&baseline, &current, 5.0);
        let summary = BenchReport::summary(&comps);
        assert!(summary.contains("No regressions"));
    }

    #[test]
    fn summary_with_regressions() {
        let baseline = vec![make_result("a", 1000.0)];
        let current = vec![make_result("a", 1200.0)];
        let comps = crate::bench_comparison::BenchComparison::compare_sets(&baseline, &current, 5.0);
        let summary = BenchReport::summary(&comps);
        assert!(summary.contains("regression"));
    }
}
