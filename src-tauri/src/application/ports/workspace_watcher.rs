// src-tauri/src/application/ports/workspace_watcher.rs
//
// Real-time workspace observation (AGENTS.md debt item #3): the desktop app
// must learn when collection/environment JSON files change outside the UI
// (external editor, another app window) instead of serving stale data.

use crate::domain::errors::DomainError;
use crate::domain::models::WorkspaceChange;
use tokio::sync::mpsc::UnboundedReceiver;

pub trait WorkspaceWatcherPort: Send + Sync {
    /// Starts watching `workspace_path` recursively. Batches of changed,
    /// relevant file paths are delivered through the returned receiver until
    /// [`stop`](Self::stop) closes it. Starting again replaces any previous
    /// watch.
    fn start(
        &self,
        workspace_path: &str,
    ) -> Result<UnboundedReceiver<WorkspaceChange>, DomainError>;

    /// Stops watching and closes the event stream.
    fn stop(&self);
}
