// src-tauri/src/infrastructure/watcher/notify_adapter.rs
//
// `notify`-backed implementation of `WorkspaceWatcherPort`.
//
// Raw filesystem events are filtered (only relevant JSON files), deduplicated
// and debounced into batches so the frontend receives one
// `workspace_changed`-style payload per burst of editor saves.

use crate::application::ports::workspace_watcher::WorkspaceWatcherPort;
use crate::domain::models::WorkspaceChange;
use crate::domain::errors::DomainError;
use notify::{RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

/// Debounce window coalescing editor save bursts.
pub const DEBOUNCE_MS: u64 = 300;

pub struct NotifyWorkspaceWatcher {
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

impl Default for NotifyWorkspaceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Only JSON workspace artifacts matter for reload semantics; security-
/// sensitive files must never leak their names into UI events.
fn is_relevant_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let lowered = name.to_lowercase();
    if !lowered.ends_with(".json") {
        return false;
    }
    let sensitive = [".vault", ".env.local", ".vault.key"];
    !sensitive.iter().any(|needle| lowered.contains(needle))
}

impl NotifyWorkspaceWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
        }
    }
}

impl WorkspaceWatcherPort for NotifyWorkspaceWatcher {
    fn start(
        &self,
        workspace_path: &str,
    ) -> Result<mpsc::UnboundedReceiver<WorkspaceChange>, DomainError> {
        let root = PathBuf::from(workspace_path);
        if !root.is_dir() {
            return Err(DomainError::ValidationError(format!(
                "workspace path '{}' is not a directory",
                workspace_path
            )));
        }

        let (tx, rx) = mpsc::unbounded_channel::<WorkspaceChange>();
        let buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let event_buffer = Arc::clone(&buffer);
        let mut raw_watcher = notify::recommended_watcher(
            move |result: Result<notify::Event, notify::Error>| {
                if let Ok(event) = result {
                    let mut guard = match event_buffer.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    for path in event.paths {
                        if is_relevant_path(&path) {
                            let as_string = path.to_string_lossy().to_string();
                            if !guard.contains(&as_string) {
                                guard.push(as_string);
                            }
                        }
                    }
                }
            },
        )
        .map_err(|error| DomainError::ConfigError(format!("failed to create watcher: {error}")))?;

        raw_watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|error| DomainError::ConfigError(format!("failed to watch '{}': {}", workspace_path, error)))?;

        // Store the handle first: dropping it would stop the watch immediately.
        {
            let mut guard = self
                .watcher
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = Some(raw_watcher);
        }

        let flush_buffer = Arc::clone(&buffer);
        let flush_seen = Arc::clone(&seen);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(DEBOUNCE_MS));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let drained: Vec<String> = match flush_buffer.lock() {
                    Ok(mut guard) => std::mem::take(&mut *guard),
                    Err(_) => Vec::new(),
                };
                if drained.is_empty() {
                    continue;
                }
                let mut unique: Vec<String> = {
                    let mut seen_guard = match flush_seen.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    drained
                        .into_iter()
                        .filter(|path| seen_guard.insert(path.clone()))
                        .collect()
                };
                unique.sort();
                if tx
                    .send(WorkspaceChange { paths: unique })
                    .is_err()
                {
                    break; // Receiver dropped: caller stopped listening.
                }
            }
        });

        Ok(rx)
    }

    fn stop(&self) {
        let mut guard = self
            .watcher
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = None; // Dropping the recommended watcher stops the OS watch.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::Duration;

    fn temp_workspace(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "tyny-watch-{tag}-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).expect("temp workspace created");
        dir
    }

    #[test]
    fn relevance_filters_json_and_sensitive_files_only() {
        assert!(is_relevant_path(Path::new("/ws/pets.json")));
        assert!(!is_relevant_path(Path::new("/ws/notes.txt")));
        assert!(!is_relevant_path(Path::new("/ws/pets.vault.enc")));
        assert!(!is_relevant_path(Path::new("/ws/.env.local.json")));
        assert!(!is_relevant_path(Path::new("/ws/pets.vault.key.json")));
    }

    #[tokio::test]
    async fn start_rejects_non_directory_paths() {
        let watcher = NotifyWorkspaceWatcher::new();
        let result = watcher.start("/definitely/not/a/dir");
        assert!(matches!(result, Err(DomainError::ValidationError(_))));
    }

    #[tokio::test]
    async fn json_edits_are_debounced_into_batches_and_vault_files_filtered() {
        let workspace = temp_workspace("events");
        let watcher = NotifyWorkspaceWatcher::new();
        let mut rx = watcher
            .start(workspace.to_string_lossy().as_ref())
            .expect("watcher started");

        // Give the OS watcher a moment to arm before touching the FS.
        tokio::time::sleep(Duration::from_millis(150)).await;

        std::fs::write(workspace.join("pets.json"), "{}").expect("write pets");
        std::fs::write(workspace.join("secrets.vault.enc"), "nope").expect("write vault");

        let mut seen_paths: Vec<String> = Vec::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        while tokio::time::Instant::now() < deadline && seen_paths.is_empty() {
            if let Ok(change) =
                tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
            {
                if let Some(change) = change {
                    seen_paths.extend(change.paths);
                    break;
                }
            }
        }

        assert!(
            seen_paths.iter().any(|path| path.ends_with("pets.json")),
            "pets.json edit must surface in a batch, got {seen_paths:?}"
        );
        assert!(
            !seen_paths.iter().any(|path| path.contains(".vault")),
            "vault files must never surface, got {seen_paths:?}"
        );

        watcher.stop();
        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[tokio::test]
    async fn stop_closes_the_event_stream() {
        let workspace = temp_workspace("stop");
        let watcher = NotifyWorkspaceWatcher::new();
        let mut rx = watcher
            .start(workspace.to_string_lossy().as_ref())
            .expect("watcher started");

        watcher.stop();

        // The sender side lives inside the debounce task; it observes the
        // closed channel on its next batch flush. Drain until closed.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            match tokio::time::timeout(Duration::from_millis(400), rx.recv()).await {
                Ok(Some(_)) => continue,
                _ => break,
            }
        }
        let _ = std::fs::remove_dir_all(&workspace);
    }
}
