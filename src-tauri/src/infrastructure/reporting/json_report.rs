use crate::domain::models::{CollectionRunReport, RequestRunResult};
use serde::Serialize;

/// JSON report envelope for headless runs. Wraps the existing
/// `CollectionRunReport` contract with pipeline-oriented metadata.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRunReport<'a> {
    pub tool: &'a str,
    pub version: &'a str,
    pub generated_at: String,
    pub collection: String,
    pub summary: ReportSummary,
    pub results: &'a [RequestRunResult],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub total_requests: usize,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
}

impl ReportSummary {
    pub fn from_report(report: &CollectionRunReport) -> Self {
        Self {
            total_requests: report.total_requests,
            total_tests: report.total_tests,
            passed_tests: report.passed_tests,
            failed_tests: report.total_tests - report.passed_tests,
        }
    }
}

impl<'a> JsonRunReport<'a> {
    pub fn new(collection_name: &str, report: &'a CollectionRunReport) -> Self {
        Self {
            tool: "tyny-pulse",
            version: env!("CARGO_PKG_VERSION"),
            generated_at: chrono::Utc::now().to_rfc3339(),
            collection: collection_name.to_string(),
            summary: ReportSummary::from_report(report),
            results: &report.results,
        }
    }
}

/// Renders the JSON report as pretty-printed JSON.
pub fn render_json(collection_name: &str, report: &CollectionRunReport) -> String {
    let envelope = JsonRunReport::new(collection_name, report);
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
    fn json_envelope_carries_metadata_and_summary() {
        let report = sample_report();
        let rendered = render_json("smoke", &report);
        assert!(rendered.contains("\"tool\": \"tyny-pulse\""));
        assert!(rendered.contains("\"totalRequests\": 2"));
        assert!(rendered.contains("\"failedTests\": 1"));
        assert!(rendered.contains("\"collection\": \"smoke\""));
        assert!(rendered.contains("\"requestName\": \"health check\""));
    }

    #[test]
    fn summary_counts_failures_as_total_minus_passed() {
        let summary = ReportSummary::from_report(&sample_report());
        assert_eq!(summary.failed_tests, 1);
        assert_eq!(summary.total_tests, 3);
    }
}
