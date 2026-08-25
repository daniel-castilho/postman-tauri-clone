// Headless CLI integration tests (Phase 18 P2 epic).
//
// Exercises the full headless path — fixture collection over a real local
// mock HTTP server, report writing, exit code mapping — without any Tauri
// GUI initialization, per the epic's "Headless Execution Test" requirement.

use tyny_pulse_lib::domain::models::Collection;
use tyny_pulse_lib::presentation::cli::{
    execute_run, is_cli_mode, parse_run_args, resolve_report_format, ReportFormat,
};
use tyny_pulse_lib::presentation::cli::{RunOptions, USAGE};
use tyny_pulse_lib::infrastructure::reporting::{json_reporter, junit_reporter};

use std::io::{Read, Write};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// Minimal HTTP/1.1 responder: GET /health -> 200 JSON, everything else
/// -> 500. Handles connections sequentially until dropped.
fn spawn_mock_server(address: &'static str) {
    std::thread::spawn(move || {
        let listener = std::net::TcpListener::bind(address).expect("mock server bind");
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut buffer = [0u8; 2048];
            let _ = stream.read(&mut buffer);
            let request = String::from_utf8_lossy(&buffer);
            let (status, body) = if request.starts_with("GET /health") {
                ("200 OK", "{\"status\":\"ok\"}")
            } else {
                ("500 Internal Server Error", "boom")
            };
            let response = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
}

fn temp_file(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tyny-cli-it-{}-{}",
        std::process::id(),
        name
    ));
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn sample_fixture_parses_into_domain_collection() {
    let raw = std::fs::read_to_string(fixture_path("sample_collection.json"))
        .expect("fixture must exist");
    let collection: Collection = serde_json::from_str(&raw).expect("fixture must deserialize");
    assert_eq!(collection.name, "Sample Collection");
    assert_eq!(collection.items.len(), 1);
}

#[test]
fn headless_run_against_mock_server_returns_exit_zero_and_writes_junit() {
    spawn_mock_server("127.0.0.1:8899");

    let report_path = temp_file("report.xml");
    let options = RunOptions {
        collection_path: fixture_path("sample_collection.json").to_string_lossy().to_string(),
        environment_path: None,
        globals_path: None,
        var_overrides: vec![],
        report_path: Some(report_path.to_string_lossy().to_string()),
        format_override: None,
    };

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let exit_code = runtime.block_on(execute_run(&options)).expect("execution succeeds");
    assert_eq!(exit_code, 0, "all fixture assertions must pass");

    let junit = std::fs::read_to_string(&report_path).expect("junit report written");
    assert!(junit.contains("<testsuites name=\"Sample Collection\""));
    assert!(junit.contains("<testcase name=\"status is 200\""));
    assert!(!junit.contains("<failure"));
}

#[test]
fn failing_assertion_maps_to_exit_one() {
    let failing_collection = r#"{
        "id": "failing", "name": "Failing", "description": null, "variables": {},
        "items": [{ "Request": {
            "id": "req-fail", "name": "broken expects 200", "description": null,
            "method": "GET", "url": "http://127.0.0.1:8899/broken",
            "headers": [], "body": null, "auth": null, "variables": {},
            "scripts": { "preRequest": "", "tests": "pm.test('status is 200', function() { if (pm.response.status !== 200) throw new Error('expected 200, got ' + pm.response.status); });" },
            "grpc_config": null
        } }]
    }"#;
    let collection_path = temp_file("failing_collection.json");
    std::fs::write(&collection_path, failing_collection).expect("write failing fixture");

    let options = RunOptions {
        collection_path: collection_path.to_string_lossy().to_string(),
        environment_path: None,
        globals_path: None,
        var_overrides: vec![],
        report_path: None,
        format_override: None,
    };

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let exit_code = runtime.block_on(execute_run(&options)).expect("execution succeeds");
    assert_eq!(exit_code, 1, "failing assertion must map to exit code 1");

    let _ = std::fs::remove_file(collection_path);
}

#[test]
fn missing_collection_file_maps_to_exit_two() {
    let options = RunOptions {
        collection_path: "/definitely/not/a/real/collection.json".to_string(),
        environment_path: None,
        globals_path: None,
        var_overrides: vec![],
        report_path: None,
        format_override: None,
    };
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let failure = runtime.block_on(execute_run(&options)).expect_err("must fail");
    assert_eq!(failure.exit_code(), 2);
}

#[test]
fn usage_errors_and_dispatch_helpers_behave_per_contract() {
    // Missing collection argument.
    assert!(parse_run_args(&[]).is_err());
    // Unknown flag surfaces the usage text.
    let unknown = parse_run_args(&["--wat".to_string()]).unwrap_err();
    assert!(unknown.contains(USAGE));
    // GUI passthrough stays untouched.
    assert!(!is_cli_mode(&["tyny-pulse".to_string(), "/file.json".to_string()]));
    assert!(is_cli_mode(&["tyny-cli".to_string(), "run".to_string(), "c.json".to_string()]));
    // Extension-based format inference matches the spec default rule.
    assert_eq!(resolve_report_format("r.xml", None), ReportFormat::JUnit);
}

#[test]
fn reporters_render_spec_compliant_documents() {
    use tyny_pulse_lib::domain::models::{CollectionRunReport, RequestRunResult, TestResult};

    let report = CollectionRunReport {
        total_requests: 1,
        total_tests: 1,
        passed_tests: 1,
        results: vec![RequestRunResult {
            request_name: "health check".to_string(),
            status: 200,
            time_ms: 8,
            tests: vec![TestResult { name: "ok".to_string(), passed: true, error: None }],
        }],
    };

    let json = json_reporter::render_json("Smoke", &report, 42);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(parsed["version"], "1.0");
    assert_eq!(parsed["summary"]["totalRequests"], 1);
    assert_eq!(parsed["summary"]["durationMs"], 42);

    let junit = junit_reporter::render_junit("Smoke", &report, 42);
    assert!(junit.contains("errors=\"0\""));
    assert!(junit.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
}
