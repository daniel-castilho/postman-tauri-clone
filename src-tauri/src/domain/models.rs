// src-tauri/src/domain/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub id: RequestId,
    pub name: String,
    pub description: Option<String>,
    pub method: HttpMethod,
    pub url: Url,
    pub headers: Vec<Header>,
    pub body: Option<Body>,
    pub auth: Option<Auth>,
    pub variables: HashMap<String, String>, // variáveis locais da request
    pub scripts: Option<HttpScripts>,       // Novo campo para automação
    pub grpc_config: Option<GrpcConfig>,   // Configuração para gRPC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpScripts {
    pub pre_request: String,
    pub tests: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, WS, GRPC, CUSTOM(String),
}

// ... (other structs)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    pub proto_path: String,
    pub service: String,
    pub method: String,
    pub metadata: Vec<GrpcMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcMetadata {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url(pub String); // Value Object simples (pode adicionar validação depois)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Body {
    Raw(String, BodyMode),
    FormData(Vec<FormField>),
    UrlEncoded(Vec<KeyValue>),
    Binary(Vec<u8>),
    GraphQL { query: String, variables: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyMode {
    Json, Xml, Text, Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub value: String,
    pub file: Option<String>, // path para arquivos
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLog {
    pub level: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Collection e Environment (simplificados para MVP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<CollectionItem>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionItem {
    Request(HttpRequest),
    Folder { name: String, description: Option<String>, items: Vec<CollectionItem> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableType {
    Public,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub initial_value: String, // Valor compartilhado
    pub current_value: String, // Valor local (não sincronizado p/ secrets)
    pub var_type: VariableType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVariables {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBundle {
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub globals: Option<GlobalVariables>,
    pub export_date: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRunResult {
    pub request_name: String,
    pub status: u16,
    pub time_ms: u64,
    pub tests: Vec<TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRunReport {
    pub total_requests: usize,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub results: Vec<RequestRunResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRule {
    pub id: String,
    pub path: String,
    pub method: HttpMethod,
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockServerStatus {
    pub is_running: bool,
    pub port: u16,
    pub active_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    pub users: u32,
    pub requests_per_user: u32,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDefinition {
    pub id: String,
    pub name: String,
    pub url: String,
    pub interval_seconds: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorReport {
    pub monitor_id: String,
    pub last_check: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub is_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberRole {
    Admin, Viewer, Editor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub user_id: String,
    pub email: String,
    pub role: MemberRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    pub id: String,
    pub resource_type: String, // "Collection", "Environment", etc.
    pub resource_id: String,
    pub operation: String, // "Create", "Update", "Delete"
    pub data: String, // JSON string do recurso
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpec {
    pub id: String,
    pub name: String,
    pub content: String,
    pub format: String, // "yaml" or "json"
    pub version: String, // "OpenAPI 3.0", "OpenAPI 3.1"
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub line: u32,
    pub message: String,
    pub severity: LintSeverity,
    pub path: String, // e.g., "paths./users.get"
}
