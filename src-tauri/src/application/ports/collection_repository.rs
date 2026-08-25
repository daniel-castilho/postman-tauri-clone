// src-tauri/src/application/ports/collection_repository.rs
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

pub trait CollectionRepositoryPort: Send + Sync {
    // Returns every collection found in the root directory of the current workspace.
    fn list_collections(&self, workspace_path: &str) -> Result<Vec<Collection>, DomainError>;
    // Saves or overwrites a specific collection file (e.g. "workspace_path/collection_id.json")
    fn save_collection(&self, workspace_path: &str, collection: &Collection) -> Result<(), DomainError>;
    // Deletes a collection by ID
    fn delete_collection(&self, workspace_path: &str, collection_id: &str) -> Result<(), DomainError>;

    // Manages the `environments.json` file in the workspace
    fn list_environments(&self, workspace_path: &str) -> Result<Vec<crate::domain::models::Environment>, DomainError>;
    fn save_environments(&self, workspace_path: &str, environments: &[crate::domain::models::Environment]) -> Result<(), DomainError>;

    // Manages the `globals.json` file in the workspace
    fn load_globals(&self, workspace_path: &str) -> Result<crate::domain::models::GlobalVariables, DomainError>;
    fn save_globals(&self, workspace_path: &str, globals: &crate::domain::models::GlobalVariables) -> Result<(), DomainError>;
}
