// src-tauri/src/application/ports/variable_resolver.rs
use crate::domain::errors::DomainError;
use std::collections::HashMap;

pub trait VariableResolverPort: Send + Sync {
    fn resolve(
        &self, 
        text: &str, 
        env_vars: &HashMap<String, String>, 
        collection_vars: &HashMap<String, String>,
        global_vars: &HashMap<String, String>,
        session_vars: &HashMap<String, String>
    ) -> Result<String, DomainError>;
}
