// src-tauri/src/application/ports/collection_repository.rs
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

pub trait CollectionRepositoryPort: Send + Sync {
    // Retorna todas as coleções encontradas no root directory do workspace atual.
    fn list_collections(&self, workspace_path: &str) -> Result<Vec<Collection>, DomainError>;
    // Salva ou sobrescreve uma coleção específica em um arquivo (ex: "workspace_path/collection_id.json")
    fn save_collection(&self, workspace_path: &str, collection: &Collection) -> Result<(), DomainError>;
    // Deleta uma coleção pelo ID
    fn delete_collection(&self, workspace_path: &str, collection_id: &str) -> Result<(), DomainError>;
    
    // Gerencia o arquivo `environments.json` no workspace
    fn list_environments(&self, workspace_path: &str) -> Result<Vec<crate::domain::models::Environment>, DomainError>;
    fn save_environments(&self, workspace_path: &str, environments: &[crate::domain::models::Environment]) -> Result<(), DomainError>;

    // Gerencia o arquivo `globals.json` no workspace
    fn load_globals(&self, workspace_path: &str) -> Result<crate::domain::models::GlobalVariables, DomainError>;
    fn save_globals(&self, workspace_path: &str, globals: &crate::domain::models::GlobalVariables) -> Result<(), DomainError>;
}
