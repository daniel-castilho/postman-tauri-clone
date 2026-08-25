// src-tauri/src/application/commands/git_tasks.rs
//
// Tauri IPC wrappers for the Git-native interface epic (P3). Thin adapters
// between the frontend Git panel and the `GitRepositoryPort`; every call is
// non-blocking because the adapter executes work on Tokio blocking threads.
use crate::application::ports::git_repository::GitRepositoryPort;
use crate::domain::errors::AppError;
use crate::domain::models::{GitFileDiffDto, GitStatusSummaryDto};
use crate::infrastructure::git::git_process_adapter::GitProcessAdapter;

use std::sync::Arc;

use tauri::State;

type GitPortState<'a> = State<'a, Arc<GitProcessAdapter>>;

#[tauri::command]
pub async fn git_get_status(
    workspace_path: String,
    port: GitPortState<'_>,
) -> Result<GitStatusSummaryDto, AppError> {
    port.get_status(&workspace_path).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn git_list_branches(
    workspace_path: String,
    port: GitPortState<'_>,
) -> Result<Vec<String>, AppError> {
    let status = port.get_status(&workspace_path).await.map_err(AppError::from)?;
    Ok(status.branches)
}

#[tauri::command]
pub async fn git_stage_file(
    workspace_path: String,
    file: String,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.stage_path(&workspace_path, &file).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn git_unstage_file(
    workspace_path: String,
    file: String,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.unstage_path(&workspace_path, &file).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn git_stage_all(
    workspace_path: String,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.stage_all(&workspace_path).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn git_commit(
    workspace_path: String,
    message: String,
    port: GitPortState<'_>,
) -> Result<String, AppError> {
    port.commit(&workspace_path, &message).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn git_push(
    workspace_path: String,
    remote: Option<String>,
    branch: Option<String>,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.push(&workspace_path, remote.as_deref(), branch.as_deref())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn git_pull(
    workspace_path: String,
    remote: Option<String>,
    branch: Option<String>,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.pull(&workspace_path, remote.as_deref(), branch.as_deref())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn git_checkout_branch(
    workspace_path: String,
    branch_name: String,
    create_if_missing: bool,
    port: GitPortState<'_>,
) -> Result<(), AppError> {
    port.checkout_branch(&workspace_path, &branch_name, create_if_missing)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn git_get_file_diff(
    workspace_path: String,
    file: String,
    staged: bool,
    port: GitPortState<'_>,
) -> Result<GitFileDiffDto, AppError> {
    port.get_diff(&workspace_path, &file, staged).await.map_err(AppError::from)
}
