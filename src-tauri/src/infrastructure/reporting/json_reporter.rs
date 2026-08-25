use crate::domain::models::{CollectionRunReport, RequestRunResult};
use serde::Serialize;

/// JSON report envelope for headless runs, per `tasks/headless-cli-spec.md`
/// section 5.1: schema version, execution summary and full request results.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRunReport<'a> {
    pub version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    pub summary: ReportSummary,
    pub results: &'a [RequestRunResult],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub total_requests: usize,
    pub passed_requests: usize,
    pub failed_requests: usize,
    pub total_assertions: usize,
    pub passed_assertions: usize,
    pub failed_assertions: usize,
    pub duration_ms: u64,
}

impl ReportSummary {
    /// A request counts as passed when none of its assertions failed
    /// (requests without scripts count as passed requests).
    pub fn from_report(report: &CollectionRunReport, duration_ms: u64) -> Self {
        let passed_requests = report
            .results
            .iter()
            .filter(|result| result.tests.iter().all(|test| test.passed))
            .count();
        Self {
            total_requests: report.total_requests,
            passed_requests,
            failed_requests: report.total_requests - passed_requests,
            total_assertions: report.total_tests,
            passed_assertions: report.passed_tests,
            failed_assertions: report.total_tests - report.passed_tests,
            duration_ms,
        }
    }
}

impl<'a> JsonRunReport<'a> {
    pub fn new(collection_name: &str, report: &'a CollectionRunReport, duration_ms: u64) -> Self {
        Self {
            version: "1.0",
            tool: Some("tyny-pulse"),
            generated_at: Some(chrono::Utc::now().to_rfc3339()),
            collection: Some(collection_name.to_string()),
            summary: ReportSummary::from_report(report, duration_ms),
            results: &report.results,
        }
    }
}

/// Renders the JSON report as pretty-printed JSON.
pub fn render_json(collection_name: &str, report: &CollectionRunReport, duration_ms: u64) -> String {
    let envelope = JsonRunReport::new(collection_name, report, duration_ms);
    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{RequestRunResult, TestResult};

    fn sample_report() -> CollectionRunReport {
        CollectionRunReport {
            total_requests: 2,
            total_tests: 3,
            passed_tests: 2,
            results: vec![
                RequestRunResult {
                    request_name: "health check".to_string(),
                    status: 200,
                    time_ms: 12,
                    tests: vec![
                        TestResult { name: "status is ok".to_string(), passed: true, error: None },
                    ],
                },
                RequestRunResult {
                    request_name: "create user".to_string(),
                    status: 500,
                    time_ms: 40,
                    tests: vec![
                        TestResult { name: "created".to_string(), passed: false, error: Some("expected 201".to_string()) },
                        TestResult { name: "fast enough".to_string(), passed: true, error: None },
                    ],
                },
            ],
        }
    }

    #[test]
    fn json_envelope_carries_spec_summary_and_metadata() {
        let report = sample_report();
        let rendered = render_json("smoke", &report, 1420);
        assert!(rendered.contains("\"version\": \"1.0\""));
        assert!(rendered.contains("\"totalRequests\": 2"));
        assert!(rendered.contains("\"passedRequests\": 1"));
        assert!(rendered.contains("\"failedRequests\": 1"));
        assert!(rendered.contains("\"totalAssertions\": 3"));
        assert!(rendered.contains("\"passedAssertions\": 2"));
        assert!(rendered.contains("\"failedAssertions\": 1"));
        assert!(rendered.contains("\"durationMs\": 1420"));
        assert!(rendered.contains("\"requestName\": \"health check\""));
    }

    #[test]
    fn zero_assertion_requests_count_as_passed() {
        let mut report = sample_report();
        report.results[1].tests.clear();
        let rendered = render_json("smoke", &report, 10);
        assert!(rendered.contains("\"passedRequests\": 2"));
        assert!(rendered.contains("\"failedRequests\": 0"));
    }

    #[test]
    fn summary_counts_failures_as_total_minus_passed() {
        let summary = ReportSummary::from_report(&sample_report(), 5);
        assert_eq!(summary.failed_assertions, 1);
        assert_eq!(summary.total_assertions, 3);
        assert_eq!(summary.duration_ms, 5);
    }
}
