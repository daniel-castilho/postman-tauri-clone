// src-tauri/src/infrastructure/persistence/fs_collection_repository.rs
use std::fs;
use std::path::{Path, PathBuf};
use crate::application::ports::collection_repository::CollectionRepositoryPort;
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

pub struct FsCollectionRepository {}

impl FsCollectionRepository {
    pub fn new() -> Self {
        Self {}
    }

    fn get_file_path(workspace_path: &str, collection_id: &str) -> PathBuf {
        Path::new(workspace_path).join(format!("{}.json", collection_id))
    }

    fn get_env_file_path(workspace_path: &str) -> PathBuf {
        Path::new(workspace_path).join("environments.json")
    }

    fn get_globals_file_path(workspace_path: &str) -> PathBuf {
        Path::new(workspace_path).join("globals.json")
    }
}

impl CollectionRepositoryPort for FsCollectionRepository {
    fn list_collections(&self, workspace_path: &str) -> Result<Vec<Collection>, DomainError> {
        let path = Path::new(workspace_path);
        if !path.exists() || !path.is_dir() {
            return Err(DomainError::PersistenceError("Workspace path is invalid or does not exist".into()));
        }

        let mut collections = Vec::new();

        let entries = fs::read_dir(path)
            .map_err(|e| DomainError::PersistenceError(format!("Error reading workspace: {}", e)))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("json") {
                    let filename = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                    if filename == "environments.json" || filename == "globals.json" {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&p) {
                        // Tenta desserializar o arquivo em uma Collection
                        if let Ok(collection) = serde_json::from_str::<Collection>(&content) {
                            collections.push(collection);
                        }
                    }
                }
            }
        }

        Ok(collections)
    }

    fn save_collection(&self, workspace_path: &str, collection: &Collection) -> Result<(), DomainError> {
        let path = Self::get_file_path(workspace_path, &collection.id);

        let json_data = serde_json::to_string_pretty(collection)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to serialize collection: {}", e)))?;

        fs::write(&path, json_data)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to write collection to disk: {}", e)))?;

        Ok(())
    }

    fn delete_collection(&self, workspace_path: &str, collection_id: &str) -> Result<(), DomainError> {
        let path = Self::get_file_path(workspace_path, collection_id);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| DomainError::PersistenceError(format!("Failed to delete collection: {}", e)))?;
        }

        Ok(())
    }

    fn list_environments(&self, workspace_path: &str) -> Result<Vec<crate::domain::models::Environment>, DomainError> {
        let path = Self::get_env_file_path(workspace_path);

        if !path.exists() {
            return Ok(vec![crate::domain::models::Environment {
                id: "env_local".into(),
                name: "Local".into(),
                variables: std::collections::HashMap::new(),
            }]);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to read environments: {}", e)))?;

        let envs: Vec<crate::domain::models::Environment> = serde_json::from_str(&content)
            .unwrap_or_else(|_| vec![]);

        Ok(envs)
    }

    fn save_environments(&self, workspace_path: &str, environments: &[crate::domain::models::Environment]) -> Result<(), DomainError> {
        let path = Self::get_env_file_path(workspace_path);

        let json_data = serde_json::to_string_pretty(environments)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to serialize environments: {}", e)))?;

        fs::write(&path, json_data)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to write environments to disk: {}", e)))?;

        Ok(())
    }

    fn load_globals(&self, workspace_path: &str) -> Result<crate::domain::models::GlobalVariables, DomainError> {
        let path = Self::get_globals_file_path(workspace_path);

        if !path.exists() {
            return Ok(crate::domain::models::GlobalVariables {
                variables: std::collections::HashMap::new(),
            });
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to read globals: {}", e)))?;

        let globals: crate::domain::models::GlobalVariables = serde_json::from_str(&content)
            .map_err(|e| DomainError::PersistenceError(format!("Invalid globals format: {}", e)))?;

        Ok(globals)
    }

    fn save_globals(&self, workspace_path: &str, globals: &crate::domain::models::GlobalVariables) -> Result<(), DomainError> {
        let path = Self::get_globals_file_path(workspace_path);

        let json_data = serde_json::to_string_pretty(globals)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to serialize globals: {}", e)))?;

        fs::write(&path, json_data)
            .map_err(|e| DomainError::PersistenceError(format!("Failed to write globals to disk: {}", e)))?;

        Ok(())
    }
}
