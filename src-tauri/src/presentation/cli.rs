use crate::application::commands::run_collection::RunCollectionUseCase;
use crate::application::commands::send_request::SendRequestUseCase;
use crate::domain::errors::DomainError;
use crate::domain::models::{Collection, CollectionRunReport, Environment, GlobalVariables};
use crate::infrastructure::environment::variable_resolver_adapter::RealVariableResolver;
use crate::infrastructure::grpc::mock_adapter::MockGrpcClientAdapter;
use crate::infrastructure::http::reqwest_adapter::ReqwestHttpClientAdapter;
use crate::infrastructure::reporting::{json_report, junit};
use crate::infrastructure::scripting::quickjs_runner::QuickJsScriptRunner;

use std::collections::HashMap;
use std::sync::Arc;

pub const USAGE: &str = "Tyny Pulse - local-first API client

USAGE:
    tyny-pulse run <collection.json> [OPTIONS]

OPTIONS:
    --env <path>          Environment JSON file to load
    --globals <path>      Global variables JSON file to load
    --var <key=value>     Override/inject an environment variable (repeatable)
    --report <path>       Write a report file (.json / .xml / .junit)
    --format <json|junit> Explicit report format when the extension is unknown
    -h, --help            Print this help and exit
    -V, --version         Print version and exit

EXIT CODES:
    0  all tests passed
    1  at least one test failed
    2  usage or input error
    3  domain/runtime error";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    JUnit,
}

impl ReportFormat {
    pub fn label(self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::JUnit => "junit",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunOptions {
    pub collection_path: String,
    pub environment_path: Option<String>,
    pub globals_path: Option<String>,
    pub var_overrides: Vec<(String, String)>,
    pub report_path: Option<String>,
    pub format_override: Option<ReportFormat>,
}

/// A parsed command-line invocation that must be handled without booting
/// the desktop shell.
#[derive(Debug)]
pub enum CliInvocation {
    Help,
    Version,
    Run(Result<RunOptions, String>),
}

/// Returns the headless invocation encoded in `argv`, or `None` when the
/// arguments do not target CLI mode (the GUI shell starts instead).
pub fn headless_command(argv: &[String]) -> Option<CliInvocation> {
    let first = argv.get(1)?;
    match first.as_str() {
        "-h" | "--help" => Some(CliInvocation::Help),
        "-V" | "--version" => Some(CliInvocation::Version),
        "run" => Some(CliInvocation::Run(parse_run_args(&argv[2..]))),
        _ => None,
    }
}

pub fn parse_run_args(arguments: &[String]) -> Result<RunOptions, String> {
    let mut collection_path: Option<String> = None;
    let mut environment_path: Option<String> = None;
    let mut globals_path: Option<String> = None;
    let mut var_overrides: Vec<(String, String)> = Vec::new();
    let mut report_path: Option<String> = None;
    let mut format_override: Option<ReportFormat> = None;

    let mut index = 0;
    while index < arguments.len() {
        let argument = arguments[index].as_str();
        match argument {
            "--env" => environment_path = Some(next_value(arguments, &mut index, "--env")?),
            "--globals" => globals_path = Some(next_value(arguments, &mut index, "--globals")?),
            "--var" => {
                let pair = next_value(arguments, &mut index, "--var")?;
                let (key, value) = pair
                    .split_once('=')
                    .ok_or_else(|| format!("--var expects <key=value>, got '{}'", pair))?;
                var_overrides.push((key.to_string(), value.to_string()));
            }
            "--report" => report_path = Some(next_value(arguments, &mut index, "--report")?),
            "--format" => {
                let value = next_value(arguments, &mut index, "--format")?;
                format_override = Some(match value.as_str() {
                    "json" => ReportFormat::Json,
                    "junit" => ReportFormat::JUnit,
                    other => return Err(format!("--format accepts json|junit, got '{}'", other)),
                });
            }
            _ if argument.starts_with('-') => {
                return Err(format!("unknown option '{}'\n\n{}", argument, USAGE))
            }
            _ => {
                if collection_path.is_some() {
                    return Err(format!(
                        "unexpected extra argument '{}' (collection already set)\n\n{}",
                        argument, USAGE
                    ));
                }
                collection_path = Some(argument.to_string());
            }
        }
        index += 1;
    }

    Ok(RunOptions {
        collection_path: collection_path.ok_or_else(|| {
            format!("missing <collection.json>\n\n{}", USAGE)
        })?,
        environment_path,
        globals_path,
        var_overrides,
        report_path,
        format_override,
    })
}

fn next_value(
    arguments: &[String],
    index: &mut usize,
    flag: &str,
) -> Result<String, String> {
    let value = arguments
        .get(*index + 1)
        .ok_or_else(|| format!("{} expects a value", flag))?;
    *index += 1;
    Ok(value.clone())
}

#[derive(Debug)]
pub enum HeadlessError {
    /// Unreadable file or unwritable report destination (exit 2).
    Io(String),
    /// Malformed JSON payload or invalid flags (exit 2).
    Parse(String),
    /// A use case surfaced a domain failure (exit 3).
    Domain(DomainError),
}

impl HeadlessError {
    pub fn message(&self) -> String {
        match self {
            HeadlessError::Io(message) => message.clone(),
            HeadlessError::Parse(message) => message.clone(),
            HeadlessError::Domain(error) => error.to_string(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            HeadlessError::Io(_) | HeadlessError::Parse(_) => 2,
            HeadlessError::Domain(_) => 3,
        }
    }
}

/// Exit policy: pipelines must fail when any assertion fails.
pub fn exit_code_for_report(report: &CollectionRunReport) -> i32 {
    if report.passed_tests < report.total_tests {
        1
    } else {
        0
    }
}

/// Infers the writer from the report file extension; an explicit
/// `--format` always wins, and unknown extensions fall back to JSON.
pub fn resolve_report_format(path: &str, explicit: Option<ReportFormat>) -> ReportFormat {
    explicit.unwrap_or_else(|| {
        let lowered = path.to_lowercase();
        if lowered.ends_with(".xml") || lowered.ends_with(".junit") {
            ReportFormat::JUnit
        } else {
            ReportFormat::Json
        }
    })
}

/// Renders the human-readable stdout summary (pure, unit-tested).
pub fn render_summary(collection_name: &str, report: &CollectionRunReport) -> String {
    let mut lines = vec![
        format!("Tyny Pulse CLI v{}", env!("CARGO_PKG_VERSION")),
        format!("Collection: {}", collection_name),
        String::new(),
    ];
    for result in &report.results {
        lines.push(format!(
            "{} -> HTTP {} ({} ms)",
            result.request_name, result.status, result.time_ms
        ));
        for test in &result.tests {
            if test.passed {
                lines.push(format!("  [PASS] {}", test.name));
            } else {
                lines.push(format!(
                    "  [FAIL] {}: {}",
                    test.name,
                    test.error.as_deref().unwrap_or("assertion failed")
                ));
            }
        }
    }
    let failed = report.total_tests.saturating_sub(report.passed_tests);
    lines.push(String::new());
    lines.push(format!(
        "Totals: {} requests | {}/{} tests passed",
        report.total_requests, report.passed_tests, report.total_tests
    ));
    if failed == 0 {
        lines.push("Result: SUCCESS".to_string());
    } else {
        lines.push(format!("Result: FAILURE ({} failed)", failed));
    }
    lines.join("\n")
}

fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, HeadlessError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| HeadlessError::Io(format!("cannot read '{}': {}", path, error)))?;
    serde_json::from_str(&raw)
        .map_err(|error| HeadlessError::Parse(format!("invalid JSON in '{}': {}", path, error)))
}

/// Executes a collection headlessly and returns the process exit code.
pub async fn execute_run(options: &RunOptions) -> Result<i32, HeadlessError> {
    let collection = load_json::<Collection>(&options.collection_path)?;
    let collection_name = collection.name.clone();

    let mut environment = match &options.environment_path {
        Some(path) => load_json::<Environment>(path)?,
        None => Environment {
            id: "cli".to_string(),
            name: "CLI".to_string(),
            variables: Vec::new(),
        },
    };
    if !options.var_overrides.is_empty() {
        let overrides: HashMap<String, String> = options.var_overrides.iter().cloned().collect();
        environment.apply_runtime_map(&overrides);
    }
    let globals = match &options.globals_path {
        Some(path) => load_json::<GlobalVariables>(path)?,
        None => GlobalVariables {
            variables: HashMap::new(),
        },
    };

    let http_client = Arc::new(ReqwestHttpClientAdapter::new());
    let variable_resolver = Arc::new(RealVariableResolver::new());
    let script_runner = Arc::new(QuickJsScriptRunner::new());
    let grpc_client = Arc::new(MockGrpcClientAdapter);
    let send_request_usecase =
        SendRequestUseCase::new(http_client, grpc_client, variable_resolver, script_runner);
    let run_collection_usecase = RunCollectionUseCase::new(send_request_usecase);

    let report = run_collection_usecase
        .execute(collection.items, &environment, &globals, &HashMap::new())
        .await
        .map_err(HeadlessError::Domain)?;

    println!("{}", render_summary(&collection_name, &report));

    if let Some(report_path) = &options.report_path {
        let format = resolve_report_format(report_path, options.format_override);
        let contents = match format {
            ReportFormat::Json => json_report::render_json(&collection_name, &report),
            ReportFormat::JUnit => junit::render_junit(&collection_name, &report),
        };
        std::fs::write(report_path, contents).map_err(|error| {
            HeadlessError::Io(format!("cannot write report '{}': {}", report_path, error))
        })?;
        println!("Report written to {} ({})", report_path, format.label());
    }

    Ok(exit_code_for_report(&report))
}

/// Handles a detected CLI invocation entirely in-process and returns the
/// process exit code.
pub fn invoke(invocation: CliInvocation) -> i32 {
    match invocation {
        CliInvocation::Help => {
            println!("{}", USAGE);
            0
        }
        CliInvocation::Version => {
            println!("tyny-pulse {}", env!("CARGO_PKG_VERSION"));
            0
        }
        CliInvocation::Run(Err(message)) => {
            eprintln!("error: {}", message);
            2
        }
        CliInvocation::Run(Ok(options)) => {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("error: cannot start async runtime: {}", error);
                    return 3;
                }
            };
            match runtime.block_on(execute_run(&options)) {
                Ok(code) => code,
                Err(failure) => {
                    eprintln!("error: {}", failure.message());
                    failure.exit_code()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn parses_full_invocation() {
        let parsed = parse_run_args(&args(&[
            "collection.json",
            "--env",
            "prod.json",
            "--globals",
            "g.json",
            "--var",
            "host=api.tyny.ca",
            "--var",
            "token=secret",
            "--report",
            "out.xml",
        ]))
        .expect("valid invocation");
        assert_eq!(parsed.collection_path, "collection.json");
        assert_eq!(parsed.environment_path.as_deref(), Some("prod.json"));
        assert_eq!(parsed.globals_path.as_deref(), Some("g.json"));
        assert_eq!(parsed.var_overrides.len(), 2);
        assert_eq!(parsed.report_path.as_deref(), Some("out.xml"));
        assert_eq!(parsed.format_override, None);
    }

    #[test]
    fn rejects_missing_collection_and_unknown_flags() {
        assert!(parse_run_args(&[]).is_err());
        assert!(parse_run_args(&args(&["a.json", "--wat"])).is_err());
        assert!(parse_run_args(&args(&["a.json", "b.json"])).is_err());
        assert!(parse_run_args(&args(&["--var", "noequals"])).is_err());
        assert!(parse_run_args(&args(&["--format", "yaml", "a.json"])).is_err());
    }

    #[test]
    fn detects_headless_dispatch_only_for_known_tokens() {
        assert!(matches!(
            headless_command(&args(&["tyny-pulse", "--help"])),
            Some(CliInvocation::Help)
        ));
        assert!(matches!(
            headless_command(&args(&["tyny-pulse", "-V"])),
            Some(CliInvocation::Version)
        ));
        assert!(headless_command(&args(&["tyny-pulse", "run", "c.json"])).is_some());
        // File associations / stray arguments must still open the GUI.
        assert!(headless_command(&args(&["tyny-pulse"])).is_none());
        assert!(headless_command(&args(&["tyny-pulse", "/path/to/file.json"])).is_none());
    }

    #[test]
    fn exit_policy_fails_pipelines_on_any_test_failure() {
        let mut report = CollectionRunReport {
            total_requests: 1,
            total_tests: 2,
            passed_tests: 2,
            results: vec![],
        };
        assert_eq!(exit_code_for_report(&report), 0);
        report.passed_tests = 1;
        assert_eq!(exit_code_for_report(&report), 1);
    }

    #[test]
    fn report_format_resolution_prefers_explicit_then_extension() {
        assert_eq!(
            resolve_report_format("out.txt", Some(ReportFormat::JUnit)),
            ReportFormat::JUnit
        );
        assert_eq!(resolve_report_format("out.junit", None), ReportFormat::JUnit);
        assert_eq!(resolve_report_format("OUT.XML", None), ReportFormat::JUnit);
        assert_eq!(resolve_report_format("out.json", None), ReportFormat::Json);
        assert_eq!(resolve_report_format("out.unknown", None), ReportFormat::Json);
    }

    #[test]
    fn summary_marks_failures_and_totals() {
        let report = CollectionRunReport {
            total_requests: 1,
            total_tests: 2,
            passed_tests: 1,
            results: vec![crate::domain::models::RequestRunResult {
                request_name: "health".to_string(),
                status: 200,
                time_ms: 9,
                tests: vec![
                    crate::domain::models::TestResult {
                        name: "status ok".to_string(),
                        passed: true,
                        error: None,
                    },
                    crate::domain::models::TestResult {
                        name: "body".to_string(),
                        passed: false,
                        error: Some("mismatch".to_string()),
                    },
                ],
            }],
        };
        let rendered = render_summary("Smoke", &report);
        assert!(rendered.contains("[PASS] status ok"));
        assert!(rendered.contains("[FAIL] body: mismatch"));
        assert!(rendered.contains("Totals: 1 requests | 1/2 tests passed"));
        assert!(rendered.contains("Result: FAILURE (1 failed)"));
    }
}
