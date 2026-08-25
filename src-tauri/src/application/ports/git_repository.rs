// src-tauri/src/application/ports/git_repository.rs
//
// Git-native interface port (P3 epic). Pure Rust trait over workspace Git
// operations; the concrete adapter lives in `infrastructure/git/` and must
// never leak into domain or application layers.
use crate::domain::errors::DomainError;
use crate::domain::models::{GitFileDiffDto, GitStatusSummaryDto};

use async_trait::async_trait;

#[async_trait]
pub trait GitRepositoryPort: Send + Sync {
    /// Full workspace status: repository detection, branch, branches,
    /// ahead/behind counters and staged/unstaged file changes. Returns a
    /// summary with `is_repository == false` instead of an error when the
    /// workspace is not a Git repository.
    async fn get_status(&self, workspace_path: &str) -> Result<GitStatusSummaryDto, DomainError>;

    async fn stage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError>;

    async fn unstage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError>;

    async fn stage_all(&self, workspace_path: &str) -> Result<(), DomainError>;

    /// Commits the current index with the given message and returns the
    /// created commit short hash.
    async fn commit(&self, workspace_path: &str, message: &str) -> Result<String, DomainError>;

    async fn push(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError>;

    async fn pull(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError>;

    async fn checkout_branch(&self, workspace_path: &str, branch_name: &str, create_if_missing: bool) -> Result<(), DomainError>;

    /// Line-level diff for one file against HEAD (`staged == false`) or
    /// against the index (`staged == true`).
    async fn get_diff(&self, workspace_path: &str, relative_path: &str, staged: bool) -> Result<GitFileDiffDto, DomainError>;
}
