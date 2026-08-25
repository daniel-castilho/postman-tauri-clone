// src-tauri/src/application/ports/http_client.rs
use crate::domain::models::{HttpRequest, HttpResponse};
use crate::domain::errors::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait HttpClientPort: Send + Sync {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, DomainError>;
}
