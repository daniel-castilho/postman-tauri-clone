// src-tauri/src/application/commands/generate_code.rs
use std::sync::Arc;
use crate::application::ports::code_generator::CodeGeneratorPort;
use crate::domain::models::HttpRequest;

#[derive(Clone)]
pub struct GenerateCodeUseCase {
    codegen_port: Arc<dyn CodeGeneratorPort>,
}

impl GenerateCodeUseCase {
    pub fn new(codegen_port: Arc<dyn CodeGeneratorPort>) -> Self {
        Self { codegen_port }
    }

    pub fn generate_js_fetch(&self, request: &HttpRequest) -> String {
        self.codegen_port.generate_js_fetch(request)
    }

    pub fn generate_node_fetch(&self, request: &HttpRequest) -> String {
        self.codegen_port.generate_node_fetch(request)
    }
}
