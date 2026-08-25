// src-tauri/src/application/commands/docs_tasks.rs
use std::sync::Arc;
use crate::application::ports::docs_generator::DocsGeneratorPort;
use crate::domain::models::Collection;
use crate::domain::errors::DomainError;

#[derive(Clone)]
pub struct DocsUseCase {
    docs_port: Arc<dyn DocsGeneratorPort>,
}

impl DocsUseCase {
    pub fn new(docs_port: Arc<dyn DocsGeneratorPort>) -> Self {
        Self { docs_port }
    }

    pub fn generate_markdown(&self, collection: &Collection) -> Result<String, DomainError> {
        self.docs_port.generate_markdown(collection)
    }
}
