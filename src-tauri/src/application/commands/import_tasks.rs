// src-tauri/src/application/commands/import_tasks.rs
use std::sync::Arc;
use crate::application::ports::import_port::ImportPort;
use crate::application::ports::collection_repository::CollectionRepository;
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

#[derive(Clone)]
pub struct ImportUseCase {
    import_port: Arc<dyn ImportPort>,
    collection_repo: Arc<dyn CollectionRepository>,
}

impl ImportUseCase {
    pub fn new(import_port: Arc<dyn ImportPort>, collection_repo: Arc<dyn CollectionRepository>) -> Self {
        Self { import_port, collection_repo }
    }

    pub fn import_openapi(&self, content: &str, workspace_path: &str) -> Result<Collection, DomainError> {
        let collection = self.import_port.parse_openapi(content)?;
        self.collection_repo.save_collection(workspace_path, &collection)?;
        Ok(collection)
    }
}
