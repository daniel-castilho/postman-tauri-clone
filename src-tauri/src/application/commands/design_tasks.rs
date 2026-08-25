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
        let mut issues = Vec::new();
        
        // Linting simulation for the MVP (Elite API standard)
        if content.contains("GET") && !content.contains("responses") {
            issues.push(LintIssue {
                line: 10, // Simulated
                message: "Every method should have at least one response defined.".into(),
                severity: LintSeverity::Error,
                path: "paths.*.get.responses".into(),
            });
        }

        if content.contains("http:") {
            issues.push(LintIssue {
                line: 2,
                message: "Prefer HTTPS over HTTP for security.".into(),
                severity: LintSeverity::Warning,
                path: "servers.url".into(),
            });
        }

        issues
    }
}
