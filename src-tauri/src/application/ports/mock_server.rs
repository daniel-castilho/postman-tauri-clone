// src-tauri/src/application/ports/mock_server.rs
use async_trait::async_trait;
use crate::domain::models::{MockRule, MockServerStatus};
use crate::domain::errors::DomainError;

#[async_trait]
pub trait MockServerPort: Send + Sync {
    async fn start(&self, port: u16, rules: Vec<MockRule>) -> Result<(), DomainError>;
    async fn stop(&self) -> Result<(), DomainError>;
    async fn get_status(&self) -> MockServerStatus;
}
