// src-tauri/src/infrastructure/git/git_process_adapter.rs
//
// Concrete `GitRepositoryPort` implementation driving the system `git`
// executable through subprocesses (user-approved alternative to the `git2`
// crate). Benefits: zero new dependencies, credentials resolved by the
// user's own HTTPS helper / SSH agent, and no additional C dependency in
// the cross-compilation chain.
//
// All blocking work runs inside `tokio::task::spawn_blocking` so async
// callers never block a UI thread during network handshakes or large
// diffs (spec section 5.2).
use crate::application::ports::git_repository::GitRepositoryPort;
use crate::domain::errors::DomainError;
use crate::domain::models::{
    GitDiffChunkDto, GitFileChangeDto, GitFileDiffDto, GitFileStatusType, GitStatusSummaryDto,
};
use crate::infrastructure::git::git_initializer;

use std::process::Command;

use async_trait::async_trait;
use tokio::task::spawn_blocking;

pub struct GitProcessAdapter;

impl GitProcessAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Runs `git <args>` inside the workspace and returns stdout on success.
    fn run_git(workspace_path: &str, args: &[&str]) -> Result<String, DomainError> {
        let output = Command::new("git")
            .current_dir(workspace_path)
            .args(args)
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    DomainError::ConfigError(
                        "Git executable not found on PATH. Install Git to use workspace collaboration.".into(),
                    )
                } else {
                    DomainError::ConfigError(format!("Failed to spawn git: {}", error))
                }
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DomainError::ConfigError(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn is_repository_sync(workspace_path: &str) -> bool {
        matches!(
            Command::new("git")
                .current_dir(workspace_path)
                .args(["rev-parse", "--is-inside-work-tree"])
                .output(),
            Ok(output) if output.status.success()
        )
    }

    fn current_branch_sync(workspace_path: &str) -> String {
        // `branch --show-current` also works on an unborn HEAD (fresh
        // `git init` with no commits), unlike `rev-parse --abbrev-ref`.
        let branch = Self::run_git(workspace_path, &["branch", "--show-current"])
            .map(|out| out.trim().to_string())
            .unwrap_or_default();
        if !branch.is_empty() {
            return branch;
        }
        Self::run_git(workspace_path, &["rev-parse", "--abbrev-ref", "HEAD"])
            .map(|out| out.trim().to_string())
            .unwrap_or_default()
    }

    /// Parses one `git status --porcelain=v1` line into zero, one or two
    /// change entries (staged and/or unstaged) for the same file.
    fn parse_porcelain_line(line: &str) -> Vec<GitFileChangeDto> {
        if line.len() < 4 {
            return vec![];
        }
        let index_status = line.as_bytes()[0] as char;
        let worktree_status = line.as_bytes()[1] as char;
        let raw_path = line[3..].to_string();
        // Rename/copy entries carry "old -> new"; the panel tracks the
        // destination path only.
        let path = match raw_path.rsplit_once(" -> ") {
            Some((_, destination)) => destination.to_string(),
            None => raw_path,
        };

        if index_status == '?' && worktree_status == '?' {
            return vec![GitFileChangeDto {
                path,
                status: GitFileStatusType::Untracked,
                is_staged: false,
            }];
        }
        if index_status == 'U' || worktree_status == 'U' {
            return vec![GitFileChangeDto {
                path,
                status: GitFileStatusType::Conflicted,
                is_staged: false,
            }];
        }

        let mut changes = vec![];
        if !matches!(index_status, ' ' | '?') {
            changes.push(GitFileChangeDto {
                path: path.clone(),
                status: Self::map_status_code(index_status),
                is_staged: true,
            });
        }
        if !matches!(worktree_status, ' ' | '?') {
            changes.push(GitFileChangeDto {
                path,
                status: Self::map_status_code(worktree_status),
                is_staged: false,
            });
        }
        changes
    }

    fn map_status_code(code: char) -> GitFileStatusType {
        match code {
            'A' | 'C' => GitFileStatusType::Added,
            'D' => GitFileStatusType::Deleted,
            'R' => GitFileStatusType::Renamed,
            _ => GitFileStatusType::Modified,
        }
    }

    fn ahead_behind_sync(workspace_path: &str) -> (u32, u32) {
        match Self::run_git(
            workspace_path,
            &["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
        ) {
            Ok(output) => {
                let mut parts = output.split_whitespace();
                let behind = parts.next().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                let ahead = parts.next().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                (ahead, behind)
            }
            Err(_) => (0, 0),
        }
    }

    fn status_summary_sync(workspace_path: &str) -> Result<GitStatusSummaryDto, DomainError> {
        if !Self::is_repository_sync(workspace_path) {
            return Ok(GitStatusSummaryDto {
                is_repository: false,
                current_branch: String::new(),
                branches: vec![],
                ahead_count: 0,
                behind_count: 0,
                files: vec![],
            });
        }

        // Security guardrail: refresh exclusions every time the panel reads
        // repository state (idempotent; see spec section 5.1).
        git_initializer::ensure_security_exclusions(workspace_path)?;

        let mut files: Vec<GitFileChangeDto> =
            Self::run_git(workspace_path, &["status", "--porcelain=v1"])?
                .lines()
                .flat_map(Self::parse_porcelain_line)
                .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path).then(b.is_staged.cmp(&a.is_staged)));

        let branches: Vec<String> = Self::run_git(workspace_path, &["branch", "--format=%(refname:short)"])?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();

        let (ahead_count, behind_count) = Self::ahead_behind_sync(workspace_path);

        Ok(GitStatusSummaryDto {
            is_repository: true,
            current_branch: Self::current_branch_sync(workspace_path),
            branches,
            ahead_count,
            behind_count,
            files,
        })
    }

    fn stage_sync(workspace_path: &str, relative_path: Option<&str>) -> Result<(), DomainError> {
        match relative_path {
            Some(path) => Self::run_git(workspace_path, &["add", "--", path]).map(|_| ()),
            None => Self::run_git(workspace_path, &["add", "-A"]).map(|_| ()),
        }
    }

    fn unstage_sync(workspace_path: &str, relative_path: &str) -> Result<(), DomainError> {
        Self::run_git(workspace_path, &["reset", "HEAD", "--", relative_path]).map(|_| ())
    }

    fn commit_sync(workspace_path: &str, message: &str) -> Result<String, DomainError> {
        if message.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Commit message must not be empty".into(),
            ));
        }
        Self::run_git(workspace_path, &["commit", "-m", message])?;
        Self::run_git(workspace_path, &["rev-parse", "--short", "HEAD"])
            .map(|hash| hash.trim().to_string())
    }

    fn push_sync(workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError> {
        let remote = remote.unwrap_or("origin").to_string();
        let branch = match branch {
            Some(explicit) => explicit.to_string(),
            None => Self::current_branch_sync(workspace_path),
        };
        if branch.is_empty() || branch == "HEAD" {
            return Err(DomainError::ValidationError(
                "Cannot push without an active branch".into(),
            ));
        }
        Self::run_git(workspace_path, &["push", &remote, &branch]).map(|_| ())
    }

    fn pull_sync(workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError> {
        let remote = remote.unwrap_or("origin").to_string();
        let branch = match branch {
            Some(explicit) => explicit.to_string(),
            None => Self::current_branch_sync(workspace_path),
        };
        if branch.is_empty() || branch == "HEAD" {
            return Err(DomainError::ValidationError(
                "Cannot pull without an active branch".into(),
            ));
        }
        Self::run_git(workspace_path, &["pull", &remote, &branch]).map(|_| ())
    }

    fn checkout_sync(
        workspace_path: &str,
        branch_name: &str,
        create_if_missing: bool,
    ) -> Result<(), DomainError> {
        if create_if_missing {
            Self::run_git(workspace_path, &["checkout", "-b", branch_name]).map(|_| ())
        } else {
            Self::run_git(workspace_path, &["checkout", branch_name]).map(|_| ())
        }
    }

    /// Parses unified diff output (`--unified=0`) into structured chunks.
    fn parse_unified_diff(relative_path: &str, diff_output: &str) -> GitFileDiffDto {
        let mut chunks: Vec<GitDiffChunkDto> = vec![];
        let mut old_line: Option<u32> = None;
        let mut new_line: Option<u32> = None;

        for line in diff_output.lines() {
            if let Some(header) = line.strip_prefix("@@") {
                // Header shape: "@@ -oldStart[,count] +newStart[,count] @@"
                let ranges = header.split("@@").next().unwrap_or("").trim();
                let mut parts = ranges.split_whitespace();
                let old_start = parts
                    .next()
                    .and_then(|part| part.trim_start_matches('-').split(',').next())
                    .and_then(|value| value.parse::<u32>().ok());
                let new_start = parts
                    .next()
                    .and_then(|part| part.trim_start_matches('+').split(',').next())
                    .and_then(|value| value.parse::<u32>().ok());
                old_line = old_start;
                new_line = new_start;
                continue;
            }
            if old_line.is_none() && new_line.is_none() {
                continue; // Skips diff/index/+++ /--- preamble lines.
            }
            let mut characters = line.chars();
            let marker = characters.next().unwrap_or(' ');
            let content = characters.as_str().to_string();
            match marker {
                '+' => {
                    chunks.push(GitDiffChunkDto {
                        old_line_number: None,
                        new_line_number: new_line,
                        content,
                        change_type: "add".to_string(),
                    });
                    new_line = new_line.map(|line| line + 1);
                }
                '-' => {
                    chunks.push(GitDiffChunkDto {
                        old_line_number: old_line,
                        new_line_number: None,
                        content,
                        change_type: "delete".to_string(),
                    });
                    old_line = old_line.map(|line| line + 1);
                }
                _ => {
                    chunks.push(GitDiffChunkDto {
                        old_line_number: old_line,
                        new_line_number: new_line,
                        content: if marker == ' ' { content } else { line.to_string() },
                        change_type: "context".to_string(),
                    });
                    old_line = old_line.map(|line| line + 1);
                    new_line = new_line.map(|line| line + 1);
                }
            }
        }

        GitFileDiffDto {
            path: relative_path.to_string(),
            chunks,
        }
    }

    fn diff_sync(
        workspace_path: &str,
        relative_path: &str,
        staged: bool,
    ) -> Result<GitFileDiffDto, DomainError> {
        let output = if staged {
            Self::run_git(workspace_path, &["diff", "--cached", "--unified=0", "--", relative_path])?
        } else {
            Self::run_git(workspace_path, &["diff", "--unified=0", "--", relative_path])?
        };
        Ok(Self::parse_unified_diff(relative_path, &output))
    }
}

impl Default for GitProcessAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitRepositoryPort for GitProcessAdapter {
    async fn get_status(&self, workspace_path: &str) -> Result<GitStatusSummaryDto, DomainError> {
        let path = workspace_path.to_string();
        spawn_blocking(move || Self::status_summary_sync(&path))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn stage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        let file = relative_path.to_string();
        spawn_blocking(move || Self::stage_sync(&path, Some(&file)))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn unstage_path(&self, workspace_path: &str, relative_path: &str) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        let file = relative_path.to_string();
        spawn_blocking(move || Self::unstage_sync(&path, &file))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn stage_all(&self, workspace_path: &str) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        spawn_blocking(move || Self::stage_sync(&path, None))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn commit(&self, workspace_path: &str, message: &str) -> Result<String, DomainError> {
        let path = workspace_path.to_string();
        let message = message.to_string();
        spawn_blocking(move || Self::commit_sync(&path, &message))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn push(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        let remote = remote.map(str::to_string);
        let branch = branch.map(str::to_string);
        spawn_blocking(move || Self::push_sync(&path, remote.as_deref(), branch.as_deref()))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn pull(&self, workspace_path: &str, remote: Option<&str>, branch: Option<&str>) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        let remote = remote.map(str::to_string);
        let branch = branch.map(str::to_string);
        spawn_blocking(move || Self::pull_sync(&path, remote.as_deref(), branch.as_deref()))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn checkout_branch(&self, workspace_path: &str, branch_name: &str, create_if_missing: bool) -> Result<(), DomainError> {
        let path = workspace_path.to_string();
        let name = branch_name.to_string();
        spawn_blocking(move || Self::checkout_sync(&path, &name, create_if_missing))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }

    async fn get_diff(&self, workspace_path: &str, relative_path: &str, staged: bool) -> Result<GitFileDiffDto, DomainError> {
        let path = workspace_path.to_string();
        let file = relative_path.to_string();
        spawn_blocking(move || Self::diff_sync(&path, &file, staged))
            .await
            .map_err(|error| DomainError::ConfigError(format!("Git task panicked: {}", error)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_untracked_and_dual_entries_from_porcelain_output() {
        // NOTE: joined explicitly because `\`-continuations would strip the
        // significant leading status column of porcelain lines.
        let output = [
            "?? new-collection.json",
            "M  staged-modified.json",
            " M unstaged-modified.json",
            "A  added.json",
            " D deleted.json",
            "R  old.json -> new-name.json",
        ]
        .join("\n")
            + "\n";
        let parsed: Vec<GitFileChangeDto> = output
            .lines()
            .flat_map(GitProcessAdapter::parse_porcelain_line)
            .collect();

        assert_eq!(parsed.len(), 6);
        assert_eq!(parsed[0].status, GitFileStatusType::Untracked);
        assert!(!parsed[0].is_staged);

        assert_eq!(parsed[1].path, "staged-modified.json");
        assert!(parsed[1].is_staged);
        assert_eq!(parsed[2].path, "unstaged-modified.json");
        assert!(!parsed[2].is_staged);

        assert_eq!(parsed[3].status, GitFileStatusType::Added);
        assert!(parsed[3].is_staged);
        assert_eq!(parsed[4].status, GitFileStatusType::Deleted);

        assert_eq!(parsed[5].path, "new-name.json");
        assert_eq!(parsed[5].status, GitFileStatusType::Renamed);
    }

    #[test]
    fn parses_conflict_marker_as_single_conflicted_entry() {
        let parsed = GitProcessAdapter::parse_porcelain_line("UU conflicted.json");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].status, GitFileStatusType::Conflicted);
    }

    #[test]
    fn unified_diff_parser_extracts_add_delete_context_chunks() {
        let diff = [
            "diff --git a/sample.json b/sample.json",
            "index 1111111..2222222 100644",
            "--- a/sample.json",
            "+++ b/sample.json",
            "@@ -1,3 +1,3 @@",
            " {",
            "-  \"old\": true",
            "+  \"new\": true",
            " }",
        ]
        .join("\n")
            + "\n";
        let parsed = GitProcessAdapter::parse_unified_diff("sample.json", &diff);

        assert_eq!(parsed.path, "sample.json");
        assert_eq!(parsed.chunks.len(), 4);
        assert_eq!(parsed.chunks[0].change_type, "context");
        assert_eq!(parsed.chunks[0].old_line_number, Some(1));
        assert_eq!(parsed.chunks[0].new_line_number, Some(1));
        assert_eq!(parsed.chunks[1].change_type, "delete");
        assert_eq!(parsed.chunks[1].content, "  \"old\": true");
        assert_eq!(parsed.chunks[1].old_line_number, Some(2));
        assert_eq!(parsed.chunks[2].change_type, "add");
        assert_eq!(parsed.chunks[2].content, "  \"new\": true");
        assert_eq!(parsed.chunks[2].new_line_number, Some(2));
        assert_eq!(parsed.chunks[3].change_type, "context");
        assert_eq!(parsed.chunks[3].old_line_number, Some(3));
        assert_eq!(parsed.chunks[3].new_line_number, Some(3));
    }

    #[test]
    fn empty_diff_yields_empty_chunk_list() {
        let parsed = GitProcessAdapter::parse_unified_diff("clean.json", "");
        assert!(parsed.chunks.is_empty());
    }
}
