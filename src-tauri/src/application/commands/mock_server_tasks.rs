// src-tauri/src/application/commands/mock_server_tasks.rs
use std::sync::Arc;
use crate::application::ports::mock_server::MockServerPort;
use crate::domain::models::{MockRule, MockServerStatus};
use crate::domain::errors::DomainError;

#[derive(Clone)]
pub struct MockServerUseCase {
    mock_port: Arc<dyn MockServerPort>,
}

impl MockServerUseCase {
    pub fn new(mock_port: Arc<dyn MockServerPort>) -> Self {
        Self { mock_port }
    }

    pub async fn start_server(&self, port: u16, rules: Vec<MockRule>) -> Result<(), DomainError> {
        self.mock_port.start(port, rules).await
    }

    pub async fn stop_server(&self) -> Result<(), DomainError> {
        self.mock_port.stop().await
    }

    pub async fn get_server_status(&self) -> MockServerStatus {
        self.mock_port.get_status().await
    }
}
