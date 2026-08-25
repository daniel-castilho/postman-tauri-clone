// src-tauri/src/application/commands/ai_tasks.rs
use std::sync::Arc;
use crate::application::ports::ai::AIPort;
use crate::domain::errors::DomainError;

#[derive(Clone)]
pub struct AITasksUseCase {
    ai_port: Arc<dyn AIPort>,
}

impl AITasksUseCase {
    pub fn new(ai_port: Arc<dyn AIPort>) -> Self {
        Self { ai_port }
    }

    pub async fn generate_tests(&self, url: &str, response_body: &str) -> Result<String, DomainError> {
        self.ai_port.generate_tests(url, response_body).await
    }

    pub async fn explain_response(&self, response_body: &str) -> Result<String, DomainError> {
        self.ai_port.explain_response(response_body).await
    }
}
