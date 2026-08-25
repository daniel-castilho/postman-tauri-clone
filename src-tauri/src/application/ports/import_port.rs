// src-tauri/src/application/ports/import_port.rs
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

pub trait ImportPort: Send + Sync {
    fn parse_openapi(&self, content: &str) -> Result<Collection, DomainError>;
}
