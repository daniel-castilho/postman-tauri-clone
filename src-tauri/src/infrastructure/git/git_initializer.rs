// src-tauri/src/infrastructure/git/git_initializer.rs
//
// Ensures every Git-backed workspace carries the Tyny Pulse security
// exclusions so encrypted vaults, master keys and machine-local overrides
// can never be committed (spec section 5.1).
use std::fs;
use std::path::Path;

use crate::domain::errors::DomainError;

pub const SECURITY_EXCLUSIONS_HEADER: &str = "# Tyny Pulse Security Exclusions";

/// Required .gitignore entries protecting secrets and machine-local files.
pub const SECURITY_EXCLUSIONS: [&str; 4] = [
    "*.vault.enc",
    ".vault.key",
    ".env.local",
    "script-libraries-local.json",
];

fn build_missing_entries_block(existing: &str) -> Option<String> {
    let missing: Vec<&str> = SECURITY_EXCLUSIONS
        .iter()
        .copied()
        .filter(|entry| !existing.lines().any(|line| line.trim() == *entry))
        .collect();
    if missing.is_empty() {
        return None;
    }
    let mut block = String::new();
    if !existing.is_empty() && !existing.ends_with('\n') {
        block.push('\n');
    }
    block.push_str(SECURITY_EXCLUSIONS_HEADER);
    block.push('\n');
    for entry in missing {
        block.push_str(entry);
        block.push('\n');
    }
    Some(block)
}

/// Appends any missing security exclusions to `<workspace>/.gitignore`,
/// creating the file when absent. Idempotent.
pub fn ensure_security_exclusions(workspace_path: &str) -> Result<(), DomainError> {
    let gitignore_path = Path::new(workspace_path).join(".gitignore");
    let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();

    match build_missing_entries_block(&existing) {
        None => Ok(()),
        Some(block) => {
            if !Path::new(workspace_path).is_dir() {
                return Err(DomainError::PersistenceError(
                    "Workspace path is invalid or does not exist".into(),
                ));
            }
            fs::write(&gitignore_path, existing + &block)
                .map_err(|error| DomainError::PersistenceError(format!("Failed to write .gitignore: {}", error)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_workspace() -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "tyny-git-init-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp workspace");
        dir
    }

    use std::path::PathBuf;

    #[test]
    fn creates_gitignore_with_all_exclusions_when_missing() {
        let workspace = temp_workspace();
        let path = workspace.to_string_lossy().to_string();

        ensure_security_exclusions(&path).expect("initializer succeeds");

        let content = fs::read_to_string(workspace.join(".gitignore")).expect("gitignore exists");
        assert!(content.contains(SECURITY_EXCLUSIONS_HEADER));
        for entry in SECURITY_EXCLUSIONS {
            assert!(content.contains(entry), "missing entry: {}", entry);
        }
        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn is_idempotent_and_preserves_user_rules() {
        let workspace = temp_workspace();
        let gitignore = workspace.join(".gitignore");
        fs::write(&gitignore, "node_modules/\ndist/\n").expect("seed gitignore");
        let path = workspace.to_string_lossy().to_string();

        ensure_security_exclusions(&path).expect("first run");
        let first = fs::read_to_string(&gitignore).unwrap();
        assert!(first.starts_with("node_modules/"));

        ensure_security_exclusions(&path).expect("second run");
        let second = fs::read_to_string(&gitignore).unwrap();
        assert_eq!(first, second, "second run must not duplicate entries");

        fs::remove_dir_all(workspace).ok();
    }
}
