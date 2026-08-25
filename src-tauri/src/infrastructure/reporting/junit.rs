use crate::domain::models::CollectionRunReport;

/// Escapes text for safe interpolation into XML element content and
/// attribute values (JUnit reports are consumed by strict XML parsers).
pub fn escape_xml(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Formats milliseconds as JUnit `time` (seconds with 3 decimals).
fn seconds_from_ms(time_ms: u64) -> String {
    format!("{:.3}", time_ms as f64 / 1000.0)
}

/// Renders a `CollectionRunReport` as a JUnit XML document:
/// one `<testsuite>` per request, one `<testcase>` per test.
pub fn render_junit(collection_name: &str, report: &CollectionRunReport) -> String {
    let failed_tests = report.total_tests - report.passed_tests;
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<testsuites name=\"{}\" tests=\"{}\" failures=\"{}\">\n",
        escape_xml(collection_name),
        report.total_tests,
        failed_tests
    ));

    for result in &report.results {
        let suite_failures = result.tests.iter().filter(|test| !test.passed).count();
        xml.push_str(&format!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{}\">\n",
            escape_xml(&result.request_name),
            result.tests.len(),
            suite_failures,
            seconds_from_ms(result.time_ms)
        ));
        for test in &result.tests {
            if test.passed {
                xml.push_str(&format!("    <testcase name=\"{}\" />\n", escape_xml(&test.name)));
            } else {
                xml.push_str(&format!(
                    "    <testcase name=\"{}\">\n      <failure message=\"{}\" />\n    </testcase>\n",
                    escape_xml(&test.name),
                    escape_xml(test.error.as_deref().unwrap_or("assertion failed"))
                ));
            }
        }
        xml.push_str("  </testsuite>\n");
    }

    xml.push_str("</testsuites>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{RequestRunResult, TestResult};

    fn sample_report() -> CollectionRunReport {
        CollectionRunReport {
            total_requests: 1,
            total_tests: 2,
            passed_tests: 1,
            results: vec![RequestRunResult {
                request_name: "create <user> & \"friends\"".to_string(),
                status: 500,
                time_ms: 1500,
                tests: vec![
                    TestResult { name: "created".to_string(), passed: false, error: Some("expected 201, got <500>".to_string()) },
                    TestResult { name: "fast".to_string(), passed: true, error: None },
                ],
            }],
        }
    }

    #[test]
    fn junit_escapes_special_characters() {
        assert_eq!(escape_xml("a<b>&\"'c"), "a&lt;b&gt;&amp;&quot;&apos;c");
    }

    #[test]
    fn junit_maps_requests_to_suites_and_tests_to_cases() {
        let rendered = render_junit("smoke", &sample_report());
        assert!(rendered.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(rendered.contains("<testsuites name=\"smoke\" tests=\"2\" failures=\"1\">"));
        assert!(rendered.contains("<testsuite name=\"create &lt;user&gt; &amp; &quot;friends&quot;\" tests=\"2\" failures=\"1\" time=\"1.500\">"));
        assert!(rendered.contains("<failure message=\"expected 201, got &lt;500&gt;\" />"));
        assert!(rendered.contains("<testcase name=\"fast\" />"));
        assert!(rendered.trim_end().ends_with("</testsuites>"));
    }

    #[test]
    fn junit_converts_milliseconds_to_seconds() {
        assert_eq!(seconds_from_ms(1500), "1.500");
        assert_eq!(seconds_from_ms(12), "0.012");
    }
}
