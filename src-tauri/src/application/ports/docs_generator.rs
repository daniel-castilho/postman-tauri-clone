// src-tauri/src/application/ports/docs_generator.rs
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

pub trait DocsGeneratorPort: Send + Sync {
    fn generate_markdown(&self, collection: &Collection) -> Result<String, DomainError>;
}
