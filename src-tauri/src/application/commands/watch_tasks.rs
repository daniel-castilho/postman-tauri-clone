// src-tauri/src/application/commands/watch_tasks.rs
//
// Thin IPC adapter for real-time workspace watching. The frontend listens to
// the `workspace-changed` event and reloads only what changed; starting a new
// watch supersedes any previous one via a generation counter.

use crate::application::ports::workspace_watcher::WorkspaceWatcherPort;
use crate::domain::errors::{AppError, DomainError};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;

/// Event name streamed to the Webview on debounced workspace changes.
pub const WORKSPACE_CHANGED_EVENT: &str = "workspace-changed";

/// Tracks which watch generation is active so stale emit loops exit when a
/// new watch starts or the user stops watching.
#[derive(Default)]
pub struct WorkspaceWatchState {
    generation: Arc<AtomicU64>,
}

impl WorkspaceWatchState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tauri::command]
pub fn start_workspace_watch(
    workspace_path: String,
    watcher: tauri::State<'_, Arc<dyn WorkspaceWatcherPort>>,
    state: tauri::State<'_, WorkspaceWatchState>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    let mut rx = watcher.start(&workspace_path).map_err(AppError::from)?;

    let generation = Arc::clone(&state.generation);
    let my_generation = generation.fetch_add(1, Ordering::SeqCst) + 1;

    tauri::async_runtime::spawn(async move {
        loop {
            if generation.load(Ordering::SeqCst) != my_generation {
                break;
            }
            match rx.recv().await {
                Some(change) => {
                    let _ = app_handle.emit(WORKSPACE_CHANGED_EVENT, change);
                }
                None => break, // Watcher stopped: channel closed.
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_workspace_watch(
    watcher: tauri::State<'_, Arc<dyn WorkspaceWatcherPort>>,
    state: tauri::State<'_, WorkspaceWatchState>,
) -> Result<(), AppError> {
    state.generation.fetch_add(1, Ordering::SeqCst);
    watcher.stop();
    Ok(())
}

// Keep DomainError in scope for future fallible variants of these commands.
const _: Option<DomainError> = None;
