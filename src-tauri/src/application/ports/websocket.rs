// src-tauri/src/application/ports/websocket.rs
use async_trait::async_trait;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait WebSocketPort: Send + Sync {
    async fn connect(&self, id: String, url: String) -> Result<(), DomainError>;
    async fn send(&self, id: String, message: String) -> Result<(), DomainError>;
    async fn disconnect(&self, id: String) -> Result<(), DomainError>;
}
