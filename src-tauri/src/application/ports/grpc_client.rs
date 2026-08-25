use crate::domain::models::{HttpRequest, HttpResponse};
use crate::domain::errors::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait GrpcClientPort: Send + Sync {
    async fn call(&self, request: HttpRequest) -> Result<HttpResponse, DomainError>;
}
