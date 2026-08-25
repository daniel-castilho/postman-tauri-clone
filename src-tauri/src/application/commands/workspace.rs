use crate::application::ports::collection_repository::CollectionRepositoryPort;
use crate::domain::models::{Collection, VariableType};
use crate::domain::errors::DomainError;
use crate::infrastructure::security::encryption::EncryptionManager;

pub struct WorkspaceUseCase {
    collection_repo: Box<dyn CollectionRepositoryPort>,
    encryption: EncryptionManager,
}

impl WorkspaceUseCase {
    pub fn new(collection_repo: Box<dyn CollectionRepositoryPort>) -> Result<Self, DomainError> {
        // Salt should be securely stored per-installation. For now we use a fixed salt
        // but this should be loaded from a secure config or OS keychain.
        // TODO: Move salt to OS-specific secure storage (Keychain/Credential Manager)
        const DEFAULT_SALT: &str = "ZGVmYXVsdHNhbHRmb3JlbmNyeXB0aW9u"; // base64 encoded default

        let encryption = EncryptionManager::new("master_key_default", DEFAULT_SALT)
            .map_err(|e| DomainError::AuthError(format!("Failed to initialize encryption: {}", e)))?;

        Ok(Self {
            collection_repo,
            encryption,
        })
    }

    pub fn load_collections(&self, workspace_path: &str) -> Result<Vec<Collection>, DomainError> {
        self.collection_repo.list_collections(workspace_path)
    }

    pub fn save_collection(&self, workspace_path: &str, collection: Collection) -> Result<(), DomainError> {
        self.collection_repo.save_collection(workspace_path, &collection)
    }

    pub fn delete_collection(&self, workspace_path: &str, collection_id: &str) -> Result<(), DomainError> {
        self.collection_repo.delete_collection(workspace_path, collection_id)
    }

    pub fn load_environments(&self, workspace_path: &str) -> Result<Vec<crate::domain::models::Environment>, DomainError> {
        let mut envs = self.collection_repo.list_environments(workspace_path)?;
        for env in &mut envs {
            for var in &mut env.variables {
                if var.var_type == VariableType::Secret && !var.current_value.is_empty() {
                    if let Ok(dec) = self.encryption.decrypt(&var.current_value) {
                        var.current_value = dec;
                    }
                }
            }
        }
        Ok(envs)
    }

    pub fn save_environments(&self, workspace_path: &str, mut environments: Vec<crate::domain::models::Environment>) -> Result<(), DomainError> {
        for env in &mut environments {
            for var in &mut env.variables {
                if var.var_type == VariableType::Secret && !var.current_value.is_empty() {
                    if let Ok(enc) = self.encryption.encrypt(&var.current_value) {
                        var.current_value = enc;
                    }
                }
            }
        }
        self.collection_repo.save_environments(workspace_path, &environments)
    }

    pub fn load_globals(&self, workspace_path: &str) -> Result<crate::domain::models::GlobalVariables, DomainError> {
        self.collection_repo.load_globals(workspace_path)
    }

    pub fn save_globals(&self, workspace_path: &str, globals: crate::domain::models::GlobalVariables) -> Result<(), DomainError> {
        self.collection_repo.save_globals(workspace_path, &globals)
    }
}
