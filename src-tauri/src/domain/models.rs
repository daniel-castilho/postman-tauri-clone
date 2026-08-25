// src-tauri/src/domain/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

// Every type in this module crosses the Tauri IPC boundary (command payloads,
// command returns or event payloads). They all derive `TS` so the React
// frontend can import generated TypeScript bindings instead of hand-written
// duplicates. Export destination is configured once via `TS_RS_EXPORT_DIR`
// (see `.cargo/config.toml`) and bindings are emitted by `cargo test`
// (`export_ts_bindings`).

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub struct RequestId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HttpRequest {
    pub id: RequestId,
    pub name: String,
    pub description: Option<String>,
    pub method: HttpMethod,
    pub url: Url,
    pub headers: Vec<Header>,
    pub body: Option<Body>,
    pub auth: Option<Auth>,
    pub variables: HashMap<String, String>, // request-local variables
    pub scripts: Option<HttpScripts>,       // New field for automation
    pub grpc_config: Option<GrpcConfig>,   // gRPC configuration
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HttpScripts {
    pub pre_request: String,
    pub tests: String,
}

// HTTP method names are intentionally uppercase acronyms (RFC 9110 wire values).
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum HttpMethod {
    GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, WS, GRPC, CUSTOM(String),
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
            HttpMethod::WS => write!(f, "WS"),
            HttpMethod::GRPC => write!(f, "GRPC"),
            HttpMethod::CUSTOM(method) => write!(f, "{}", method),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GrpcConfig {
    pub proto_path: String,
    pub service: String,
    pub method: String,
    pub metadata: Vec<GrpcMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GrpcMetadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Url(pub String); // Simple Value Object (validation may be added later)

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Body {
    Raw(String, BodyMode),
    FormData(Vec<FormField>),
    UrlEncoded(Vec<KeyValue>),
    Binary(Vec<u8>),
    GraphQL { query: String, variables: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum BodyMode {
    Json, Xml, Text, Html,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FormField {
    pub key: String,
    pub value: String,
    pub file: Option<String>, // file path
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

// Variant names mirror the serialized auth type tags on the wire (`type: "NoAuth"`, ...);
// renaming them would be a breaking IPC/persistence change.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum Auth {
    NoAuth,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String, in_header: bool },
    OAuth2 {
        access_token: String,
        header_prefix: Option<String>
    },
    AWSSig4 {
        access_key: String,
        secret_key: String,
        region: String,
        service: String,
        session_token: Option<String>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLog {
    pub level: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<Header>,
    pub body: Option<String>,
    pub time_ms: u64,
    pub size_bytes: usize,
    pub tests_results: Vec<TestResult>,
    pub logs: Vec<ScriptLog>,
}

/// Named IPC output of the `send_request` command. Replaces the previous
/// anonymous tuple so the full response contract is exported to TypeScript.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SendRequestOutput {
    pub response: HttpResponse,
    pub environment: Environment,
    pub globals: GlobalVariables,
    pub session_vars: HashMap<String, String>,
}

// Collection and Environment (simplified for the MVP)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<CollectionItem>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CollectionItem {
    Request(Box<HttpRequest>),
    Folder { name: String, description: Option<String>, items: Vec<CollectionItem> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum VariableType {
    Public,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EnvironmentVariable {
    pub key: String,
    pub initial_value: String, // Shared value
    pub current_value: String, // Local-only value (not synced for secrets)
    pub var_type: VariableType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}

impl Environment {
    /// Flattens the environment variables into the runtime key/value map
    /// consumed by variable resolution and scripting. The current value takes
    /// precedence over the initial value when it is non-empty.
    pub fn to_runtime_map(&self) -> HashMap<String, String> {
        self.variables
            .iter()
            .filter(|variable| variable.enabled)
            .map(|variable| {
                let value = if variable.current_value.is_empty() {
                    variable.initial_value.clone()
                } else {
                    variable.current_value.clone()
                };
                (variable.key.clone(), value)
            })
            .collect()
    }

    /// Writes runtime mutations (e.g. `pm.environment.set`) back into the
    /// structured variable list, creating new entries for unknown keys.
    pub fn apply_runtime_map(&mut self, values: &HashMap<String, String>) {
        for (key, value) in values {
            if let Some(variable) = self.variables.iter_mut().find(|v| &v.key == key) {
                variable.current_value = value.clone();
            } else {
                self.variables.push(EnvironmentVariable {
                    key: key.clone(),
                    initial_value: value.clone(),
                    current_value: value.clone(),
                    var_type: VariableType::Public,
                    enabled: true,
                });
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GlobalVariables {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkspaceBundle {
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub globals: Option<GlobalVariables>,
    pub export_date: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RequestRunResult {
    pub request_name: String,
    pub status: u16,
    pub time_ms: u64,
    pub tests: Vec<TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRunReport {
    pub total_requests: usize,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub results: Vec<RequestRunResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MockRule {
    pub id: String,
    pub path: String,
    pub method: HttpMethod,
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MockServerStatus {
    pub is_running: bool,
    pub port: u16,
    pub active_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoadTestConfig {
    pub users: u32,
    pub requests_per_user: u32,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LoadTestReport {
    pub total_requests: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub avg_time_ms: f64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub p95_time_ms: u64,
    pub requests_per_second: f64,
}

// --- Load testing engine (P4 epic) ---

/// Configuration for the streaming Tokio load testing engine.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LoadTestConfigDto {
    /// Target HTTP request executed by every virtual user.
    pub target_request: HttpRequest,
    /// Concurrent virtual users (1-500).
    pub virtual_users: u32,
    /// Total test duration in seconds (1-3600).
    pub duration_seconds: u64,
    /// Time window used to gradually scale up active virtual users.
    pub ramp_up_seconds: u64,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Latency distribution snapshot computed over all collected samples.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LatencyPercentilesDto {
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

/// One bucket of the HTTP status code breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StatusCodeCountDto {
    /// HTTP status code, or `0` for transport errors/timeouts.
    pub code: u16,
    pub count: u64,
}

/// Sampled metrics snapshot streamed to the Webview every 200ms while a
/// load test runs. The final event of a run carries `is_finished: true`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LoadTestProgressEventDto {
    pub test_id: String,
    pub elapsed_seconds: f64,
    pub active_vus: u32,
    pub current_rps: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub bytes_per_second: f64,
    pub percentiles: LatencyPercentilesDto,
    pub status_codes: Vec<StatusCodeCountDto>,
    pub is_finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonitorDefinition {
    pub id: String,
    pub name: String,
    pub url: String,
    pub interval_seconds: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MonitorReport {
    pub monitor_id: String,
    pub last_check: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub is_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MemberRole {
    Admin, Viewer, Editor,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkspaceMember {
    pub user_id: String,
    pub email: String,
    pub role: MemberRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SyncChange {
    pub id: String,
    pub resource_type: String, // "Collection", "Environment", etc.
    pub resource_id: String,
    pub operation: String, // "Create", "Update", "Delete"
    pub data: String, // JSON string of the resource
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DesignSpec {
    pub id: String,
    pub name: String,
    pub content: String,
    pub format: String, // "yaml" or "json"
    pub version: String, // "OpenAPI 3.0", "OpenAPI 3.1"
    pub last_modified: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LintIssue {
    pub line: u32,
    pub message: String,
    pub severity: LintSeverity,
    pub path: String, // e.g., "paths./users.get"
}

/// Metadata for a script library exposed to sandbox scripts via
/// `require('name')`. `enabled` mirrors the workspace-level
/// `script-libraries.json` settings file.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLibraryInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

// --- Git-native interface (P3 epic) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum GitFileStatusType {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Conflicted,
}

/// One workspace file change as shown in the Git panel. The same relative
/// path can appear twice when it has both staged and unstaged modifications.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitFileChangeDto {
    pub path: String,
    pub status: GitFileStatusType,
    pub is_staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitStatusSummaryDto {
    pub is_repository: bool,
    pub current_branch: String,
    pub branches: Vec<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub files: Vec<GitFileChangeDto>,
}

/// One diff hunk line. `change_type` is "add", "delete" or "context".
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitDiffChunkDto {
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
    pub content: String,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitFileDiffDto {
    pub path: String,
    pub chunks: Vec<GitDiffChunkDto>,
}

// --- Shared run-report rendering (GUI + headless CLI) ---

/// Output formats supported by the shared run-report renderers.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RunReportFormat {
    Html,
    Markdown,
}
