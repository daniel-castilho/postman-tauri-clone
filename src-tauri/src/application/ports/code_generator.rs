// src-tauri/src/application/ports/code_generator.rs
use crate::domain::models::HttpRequest;

pub trait CodeGeneratorPort: Send + Sync {
    fn generate_js_fetch(&self, request: &HttpRequest) -> String;
    fn generate_node_fetch(&self, request: &HttpRequest) -> String;
}
