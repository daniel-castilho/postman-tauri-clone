// src-tauri/src/infrastructure/persistence/fs_script_settings_repository.rs
//
// Persists the workspace-level script library enable/disable settings as a
// Git-friendly JSON file (`script-libraries.json`). A missing or malformed
// file means "everything enabled" (zero-config default).
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::errors::DomainError;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct ScriptLibrarySettings {
    #[serde(default)]
    disabled: Vec<String>,
}

pub struct FsScriptSettingsRepository;

impl FsScriptSettingsRepository {
    pub fn new() -> Self {
        Self
    }

    fn file_path(workspace_path: &str) -> PathBuf {
        Path::new(workspace_path).join("script-libraries.json")
    }

    /// Loads the disabled library names; any failure degrades to an empty
    /// list so scripts keep working even with a corrupted settings file.
    pub fn load_disabled(&self, workspace_path: &str) -> Vec<String> {
        let path = Self::file_path(workspace_path);
        let Ok(content) = fs::read_to_string(&path) else {
            return Vec::new();
        };
        serde_json::from_str::<ScriptLibrarySettings>(&content)
            .map(|settings| settings.disabled)
            .unwrap_or_default()
    }

    pub fn save_disabled(
        &self,
        workspace_path: &str,
        disabled: &[String],
    ) -> Result<(), DomainError> {
        if !Path::new(workspace_path).is_dir() {
            return Err(DomainError::PersistenceError(
                "Workspace path is invalid or does not exist".into(),
            ));
        }
        let settings = ScriptLibrarySettings {
            disabled: disabled.to_vec(),
        };
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to serialize script library settings: {}", e)))?;
        fs::write(Self::file_path(workspace_path), json)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to write script library settings: {}", e)))?;
        Ok(())
    }
}

impl Default for FsScriptSettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn temp_workspace() -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "tyny-script-settings-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp workspace");
        dir
    }

    #[test]
    fn missing_file_means_nothing_disabled() {
        let workspace = temp_workspace();
        let repository = FsScriptSettingsRepository::new();
        assert!(repository.load_disabled(workspace.to_str().unwrap()).is_empty());
        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn round_trips_disabled_list() {
        let workspace = temp_workspace();
        let repository = FsScriptSettingsRepository::new();
        let path = workspace.to_str().unwrap().to_string();

        repository
            .save_disabled(&path, &["dayjs".to_string(), "uuid".to_string()])
            .expect("save settings");
        let disabled = repository.load_disabled(&path);

        assert_eq!(disabled, vec!["dayjs".to_string(), "uuid".to_string()]);
        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn corrupted_file_degrades_to_empty() {
        let workspace = temp_workspace();
        let path = workspace.to_str().unwrap().to_string();
        fs::write(Path::new(workspace.as_path()).join("script-libraries.json"), "{oops")
            .expect("write corrupt file");

        let repository = FsScriptSettingsRepository::new();
        assert!(repository.load_disabled(&path).is_empty());
        fs::remove_dir_all(workspace).ok();
    }

    #[test]
    fn rejects_invalid_workspace_path() {
        let repository = FsScriptSettingsRepository::new();
        let result = repository.save_disabled("/definitely/not/a/real/dir", &[]);
        assert!(result.is_err());
    }
}
