use crate::BenchResult;

/// Parses `cargo bench` output into `BenchResult` structs.
///
/// Supports the standard libtest benchmark format:
/// ```text
/// test bench_foo      ... bench:       2,345 ns/iter (+/- 120)
/// ```
pub struct BenchParser;

impl BenchParser {
    /// Parse a full `cargo bench` output string into a list of results.
    pub fn parse(output: &str) -> Vec<BenchResult> {
        output
            .lines()
            .filter_map(|line| Self::parse_line(line))
            .collect()
    }

    /// Parse a single benchmark output line.
    pub fn parse_line(line: &str) -> Option<BenchResult> {
        // Match: test <name> ... bench: <time> ns/iter (+/- <stddev>)
        // or:    test <name> ... bench: <time> ns/iter (+/- <stddev>) = <iterations>
        let line = line.trim();

        if !line.contains("bench:") {
            return None;
        }

        // Extract name: everything between "test " and " ... bench:"
        let name = Self::extract_name(line)?;

        // Extract the timing value after "bench:"
        let bench_part = line.split("bench:").nth(1)?;
        let bench_part = bench_part.trim();

        // Parse "2,345 ns/iter (+/- 120)" or "2345 ns/iter (+/- 120)"
        let parts: Vec<&str> = bench_part.split("ns/iter").collect();
        if parts.is_empty() {
            return None;
        }

        let mean_str = parts[0].trim().replace(',', "");
        let mean_ns: f64 = mean_str.parse().ok()?;

        // Parse stddev from "(+/- 120)"
        let stddev_ns = parts
            .get(1)
            .and_then(|s| Self::extract_stddev(s))
            .unwrap_or(0.0);

        // We don't get median or iterations from standard output
        Some(BenchResult::new(name, mean_ns, mean_ns, stddev_ns, 0))
    }

    fn extract_name(line: &str) -> Option<String> {
        let start = line.find("test ")? + 5;
        let end = line.find(" ... bench:")?;
        if end <= start {
            return None;
        }
        Some(line[start..end].to_string())
    }

    fn extract_stddev(s: &str) -> Option<f64> {
        let start = s.find("(+/- ")? + 5;
        let rest = &s[start..];
        let end = rest.find(')')?;
        let num_str = rest[..end].replace(',', "");
        num_str.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_line() {
        let result = BenchParser::parse_line(
            "test bench_encode ... bench:       2,345 ns/iter (+/- 120)"
        ).unwrap();
        assert_eq!(result.name, "bench_encode");
        assert_eq!(result.mean_ns, 2345.0);
        assert_eq!(result.stddev_ns, 120.0);
    }

    #[test]
    fn parse_line_no_commas() {
        let result = BenchParser::parse_line(
            "test bench_decode ... bench:         987 ns/iter (+/- 45)"
        ).unwrap();
        assert_eq!(result.name, "bench_decode");
        assert_eq!(result.mean_ns, 987.0);
        assert_eq!(result.stddev_ns, 45.0);
    }

    #[test]
    fn parse_full_output() {
        let output = r#"
running 3 tests
test bench_foo ... bench:       1,000 ns/iter (+/- 50)
test bench_bar ... bench:       2,500 ns/iter (+/- 100)
test bench_baz ... bench:       5,000 ns/iter (+/- 200)

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out
"#;
        let results = BenchParser::parse(output);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "bench_foo");
        assert_eq!(results[1].name, "bench_bar");
        assert_eq!(results[2].name, "bench_baz");
    }

    #[test]
    fn parse_ignores_non_bench_lines() {
        assert!(BenchParser::parse_line("test bench_foo ... ok").is_none());
        assert!(BenchParser::parse_line("running 3 tests").is_none());
    }

    #[test]
    fn parse_large_numbers() {
        let result = BenchParser::parse_line(
            "test bench_heavy ... bench:  12,345,678 ns/iter (+/- 1,234)"
        ).unwrap();
        assert_eq!(result.mean_ns, 12345678.0);
        assert_eq!(result.stddev_ns, 1234.0);
    }
}
