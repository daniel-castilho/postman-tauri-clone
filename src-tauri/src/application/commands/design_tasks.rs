use crate::application::ports::design_repository::DesignRepositoryPort;
use crate::domain::models::{DesignSpec, LintIssue, LintSeverity};
use crate::domain::errors::DomainError;
use chrono::Utc;
use uuid::Uuid;

pub struct DesignUseCase {
    repo: Box<dyn DesignRepositoryPort>,
}

impl DesignUseCase {
    pub fn new(repo: Box<dyn DesignRepositoryPort>) -> Self {
        Self { repo }
    }

    pub fn list_designs(&self, workspace_path: &str) -> Result<Vec<DesignSpec>, DomainError> {
        self.repo.list_designs(workspace_path)
    }

    pub fn create_design(&self, workspace_path: &str, name: String, format: String) -> Result<DesignSpec, DomainError> {
        let design = DesignSpec {
            id: Uuid::new_v4().to_string(),
            name,
            content: String::new(),
            format,
            version: "OpenAPI 3.0".into(),
            last_modified: Utc::now().to_rfc3339(),
        };
        self.repo.save_design(workspace_path, &design)?;
        Ok(design)
    }

    pub fn save_design(&self, workspace_path: &str, mut design: DesignSpec) -> Result<(), DomainError> {
        design.last_modified = Utc::now().to_rfc3339();
        self.repo.save_design(workspace_path, &design)
    }

    pub fn delete_design(&self, workspace_path: &str, design_id: &str) -> Result<(), DomainError> {
        self.repo.delete_design(workspace_path, design_id)
    }

    pub fn lint_spec(&self, content: &str) -> Vec<LintIssue> {
        let Some(document) = parse_spec_document(content) else {
            return vec![LintIssue {
                line: 1,
                message: "Document is neither valid JSON nor valid YAML.".into(),
                severity: LintSeverity::Error,
                path: "(root)".into(),
            }];
        };

        let mut issues = Vec::new();
        let version = document
            .get("openapi")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();

        match () {
            _ if version.starts_with("3.0") => lint_common(&document, content, &mut issues),
            _ if version.starts_with("3.1") => {
                lint_common(&document, content, &mut issues);
                lint_openapi_31(&document, content, &mut issues);
            }
            _ => issues.push(LintIssue {
                line: find_line_number(content, "\"openapi\""),
                message: format!(
                    "Unrecognized OpenAPI version '{}'. SpecHub validates 3.0 and 3.1 documents.",
                    if version.is_empty() { "<missing>" } else { &version }
                ),
                severity: LintSeverity::Error,
                path: "openapi".into(),
            }),
        }

        issues
    }
}

// --- Structural linting ------------------------------------------------------

/// Rules shared by every supported OpenAPI dialect (3.0 and 3.1).
fn lint_common(document: &serde_json::Value, content: &str, issues: &mut Vec<LintIssue>) {
    // Every operation should declare at least one response.
    if let Some(paths) = document.get("paths").and_then(serde_json::Value::as_object) {
        for (path, item) in paths {
            let Some(methods) = item.as_object() else {
                continue;
            };
            for (method, operation) in methods {
                let method = method.to_ascii_lowercase();
                if !matches!(
                    method.as_str(),
                    "get" | "post" | "put" | "patch" | "delete" | "options" | "head" | "trace"
                ) {
                    continue;
                }
                let has_responses = operation
                    .get("responses")
                    .and_then(serde_json::Value::as_object)
                    .is_some_and(|responses| !responses.is_empty());
                if !has_responses {
                    issues.push(LintIssue {
                        line: find_line_number(content, &format!("\"{path}\"")),
                        message: "Every method should have at least one response defined.".into(),
                        severity: LintSeverity::Error,
                        path: format!("paths.{path}.{method}.responses"),
                    });
                }
            }
        }
    }

    // Prefer HTTPS server URLs.
    if let Some(servers) = document.get("servers").and_then(serde_json::Value::as_array) {
        for server in servers {
            if let Some(url) = server.get("url").and_then(serde_json::Value::as_str) {
                if url.starts_with("http://") {
                    issues.push(LintIssue {
                        line: find_line_number(content, url),
                        message: "Prefer HTTPS over HTTP for security.".into(),
                        severity: LintSeverity::Warning,
                        path: "servers.url".into(),
                    });
                }
            }
        }
    }
}

/// Object keys whose presence marks a value as a JSON Schema node.
const SCHEMA_MARKER_KEYS: [&str; 9] = [
    "type",
    "properties",
    "items",
    "allOf",
    "oneOf",
    "anyOf",
    "$ref",
    "not",
    "patternProperties",
];

/// Rules exclusive to the OpenAPI 3.1 JSON Schema dialect (2020-12).
fn lint_openapi_31(document: &serde_json::Value, content: &str, issues: &mut Vec<LintIssue>) {
    // 3.1 requires schemas to declare their JSON Schema dialect explicitly.
    match document.get("jsonSchemaDialect").and_then(|v| v.as_str()) {
        None => issues.push(LintIssue {
            line: find_line_number(content, "\"openapi\""),
            message: "Declare `jsonSchemaDialect` (e.g. the OAS 3.1 dialect meta-schema URL)."
                .into(),
            severity: LintSeverity::Warning,
            path: "jsonSchemaDialect".into(),
        }),
        Some(dialect)
            if !dialect
                .starts_with("https://spec.openapis.org/oas/3.1/dialect") =>
        {
            issues.push(LintIssue {
                line: find_line_number(content, dialect),
                message: "Unknown `jsonSchemaDialect`; expected the OAS 3.1 (2020-12) dialect."
                    .into(),
                severity: LintSeverity::Warning,
                path: "jsonSchemaDialect".into(),
            });
        }
        _ => {}
    }

    if let Some(root) = document.as_object() {
        for (key, value) in root {
            visit_schema_node(key, value, content, issues);
        }
    }
}

/// Recursively inspects a value tree, flagging 3.0-era schema keywords that
/// changed meaning or were removed in the 3.1 dialect.
fn visit_schema_node(key: &str, node: &serde_json::Value, content: &str, issues: &mut Vec<LintIssue>) {
    let path_prefix = if key.is_empty() {
        String::new()
    } else {
        format!("{key}.")
    };

    match node {
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                visit_schema_node(&format!("{key}/{index}"), item, content, issues);
            }
        }
        serde_json::Value::Object(map) => {
            let is_schema = SCHEMA_MARKER_KEYS.iter().any(|marker| map.contains_key(*marker));

            if is_schema {
                // 3.0-ism: `nullable` was removed in favour of type arrays.
                if map.get("nullable") == Some(&serde_json::Value::Bool(true)) {
                    issues.push(LintIssue {
                        line: find_line_number(content, "\"nullable\""),
                        message:
                            "`nullable` was removed in OpenAPI 3.1; use a type array such as [\"string\", \"null\"]."
                                .into(),
                        severity: LintSeverity::Error,
                        path: format!("{path_prefix}nullable"),
                    });
                }

                // 2020-12: `exclusiveMinimum` is now a number, not a boolean modifier.
                if matches!(
                    map.get("exclusiveMinimum"),
                    Some(serde_json::Value::Bool(_))
                ) {
                    issues.push(LintIssue {
                        line: find_line_number(content, "\"exclusiveMinimum\""),
                        message:
                            "`exclusiveMinimum` must be a number in OpenAPI 3.1 (boolean form removed)."
                                .into(),
                        severity: LintSeverity::Error,
                        path: format!("{path_prefix}exclusiveMinimum"),
                    });
                }

                // Single `example` moved to the plural `examples` array.
                if map.contains_key("example") {
                    issues.push(LintIssue {
                        line: find_line_number(content, "\"example\""),
                        message: "Use `examples` (array) instead of the singular `example` keyword."
                            .into(),
                        severity: LintSeverity::Warning,
                        path: format!("{path_prefix}example"),
                    });
                }

                // 2020-12 extension keywords with limited tooling support.
                for dynamic in ["$dynamicRef", "$dynamicAnchor"] {
                    if map.contains_key(dynamic) {
                        issues.push(LintIssue {
                            line: find_line_number(content, dynamic),
                            message: format!(
                                "`{dynamic}` has limited tooling support across OpenAPI ecosystems."
                            ),
                            severity: LintSeverity::Warning,
                            path: format!("{path_prefix}{dynamic}"),
                        });
                    }
                }
            }

            for (child_key, child) in map {
                visit_schema_node(&format!("{path_prefix}{child_key}"), child, content, issues);
            }
        }
        _ => {}
    }
}

/// Parses a spec document accepting both JSON and YAML inputs.
fn parse_spec_document(content: &str) -> Option<serde_json::Value> {
    serde_json::from_str(content)
        .ok()
        .or_else(|| serde_yaml::from_str::<serde_json::Value>(content).ok())
}

/// Best-effort 1-based line lookup used to point authors at the offending spot.
fn find_line_number(content: &str, needle: &str) -> u32 {
    content
        .lines()
        .position(|line| line.contains(needle))
        .map_or(1, |index| index as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::fs_design_repository::FsDesignRepository;

    fn service() -> DesignUseCase {
        DesignUseCase::new(Box::new(FsDesignRepository))
    }

    #[test]
    fn lint_30_document_keeps_legacy_rules() {
        let content = r#"{
            "openapi": "3.0.3",
            "servers": [{ "url": "http://api.example.com" }],
            "paths": {
                "/pets": { "get": { "summary": "List pets" } }
            }
        }"#;
        let issues = service().lint_spec(content);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].severity, LintSeverity::Error);
        assert_eq!(issues[0].path, "paths./pets.get.responses");
        assert_eq!(issues[1].severity, LintSeverity::Warning);
        assert_eq!(issues[1].path, "servers.url");
    }

    #[test]
    fn lint_rejects_unparseable_documents() {
        let issues = service().lint_spec("this is not a document {{{");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, LintSeverity::Error);
    }

    #[test]
    fn lint_unknown_version_is_flagged() {
        let content = r#"{ "openapi": "2.0", "paths": {} }"#;
        let issues = service().lint_spec(content);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].path, "openapi");
        assert_eq!(issues[0].severity, LintSeverity::Error);
    }

    #[test]
    fn lint_valid_31_document_has_no_issues() {
        let content = r#"{
            "openapi": "3.1.0",
            "jsonSchemaDialect": "https://spec.openapis.org/oas/3.1/dialect/base",
            "servers": [{ "url": "https://api.example.com" }],
            "paths": {
                "/pets": { "get": { "responses": { "200": { "description": "ok" } } } }
            },
            "components": {
                "schemas": {
                    "Pet": { "type": "object", "properties": { "name": { "type": ["string", "null"] } } }
                }
            }
        }"#;
        let issues = service().lint_spec(content);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn lint_31_flags_missing_json_schema_dialect() {
        let content = r#"{ "openapi": "3.1.0", "paths": {} }"#;
        let issues = service().lint_spec(content);
        assert!(issues.iter().any(|issue| issue.path == "jsonSchemaDialect"
            && issue.severity == LintSeverity::Warning));
    }

    #[test]
    fn lint_31_flags_wrong_dialect_url() {
        let content =
            r#"{ "openapi": "3.1.0", "jsonSchemaDialect": "http://json-schema.org/draft-07/schema#" }"#;
        let issues = service().lint_spec(content);
        assert!(
            issues
                .iter()
                .any(|issue| issue.path == "jsonSchemaDialect"
                    && issue.message.contains("2020-12"))
        );
    }

    #[test]
    fn lint_31_flags_nullable_with_line_number() {
        let content = r#"{
            "openapi": "3.1.0",
            "components": {
                "schemas": {
                    "Pet": { "type": "object", "properties": { "name": { "type": "string", "nullable": true } } }
                }
            }
        }"#;
        let issues = service().lint_spec(content);
        let nullable: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("nullable"))
            .collect();
        assert_eq!(nullable.len(), 1);
        assert_eq!(nullable[0].severity, LintSeverity::Error);
        assert!(nullable[0].path.ends_with("nullable"));
        // The probe sits on line 5 of the document.
        assert_eq!(nullable[0].line, 5);
    }

    #[test]
    fn lint_31_flags_boolean_exclusive_minimum() {
        let content = r#"{
            "openapi": "3.1.0",
            "components": {
                "schemas": {
                    "Age": { "type": "integer", "minimum": 0, "exclusiveMinimum": true }
                }
            }
        }"#;
        let issues = service().lint_spec(content);
        assert!(issues.iter().any(|issue| issue
            .message
            .contains("`exclusiveMinimum` must be a number")));
    }

    #[test]
    fn lint_31_flags_singular_example() {
        let content = r#"{
            "openapi": "3.1.0",
            "components": {
                "schemas": {
                    "Pet": { "type": "object", "example": { "name": "Rex" } }
                }
            }
        }"#;
        let issues = service().lint_spec(content);
        assert!(issues
            .iter()
            .any(|issue| issue.path.ends_with("example")
                && issue.severity == LintSeverity::Warning));
    }

    #[test]
    fn lint_31_flags_dynamic_ref_keywords() {
        let content = r#"{
            "openapi": "3.1.0",
            "components": {
                "schemas": {
                    "Tree": { "$dynamicAnchor": "node", "type": "object" }
                }
            }
        }"#;
        let issues = service().lint_spec(content);
        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("$dynamicAnchor")));
    }

    #[test]
    fn lint_accepts_yaml_31_documents() {
        let content = "\
openapi: 3.1.0
jsonSchemaDialect: https://spec.openapis.org/oas/3.1/dialect/base
paths:
  /pets:
    get:
      responses:
        '200': { description: ok }
";
        let issues = service().lint_spec(content);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn design_use_case_smoke_via_public_api() {
        // Ensures the public entry point stays wired end-to-end.
        let issues = service().lint_spec("{ not json }");
        assert!(!issues.is_empty());
    }
}
