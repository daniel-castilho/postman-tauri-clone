// src-tauri/src/application/ports/ai.rs
use async_trait::async_trait;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait AIPort: Send + Sync {
    async fn generate_tests(&self, url: &str, response_body: &str) -> Result<String, DomainError>;
    async fn explain_response(&self, response_body: &str) -> Result<String, DomainError>;
}
