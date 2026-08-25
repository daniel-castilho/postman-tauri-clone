// src-tauri/src/infrastructure/environment/variable_resolver_adapter.rs
use crate::application::ports::variable_resolver::VariableResolverPort;
use crate::domain::errors::DomainError;
use std::collections::HashMap;

pub struct RealVariableResolver {}

impl RealVariableResolver {
    pub fn new() -> Self {
        Self {}
    }

    fn substitute(text: &str, vars: &HashMap<String, String>) -> String {
        let mut result = text.to_string();
        
        // 1. Variáveis dinâmicas do sistema (Postman style)
        if result.contains("{{$guid}}") {
            result = result.replace("{{$guid}}", &uuid::Uuid::new_v4().to_string());
        }
        if result.contains("{{$timestamp}}") {
            result = result.replace("{{$timestamp}}", &chrono::Utc::now().timestamp().to_string());
        }
        if result.contains("{{$randomInt}}") {
            let rnd: u32 = rand::random::<u16>() as u32; // Simples
            result = result.replace("{{$randomInt}}", &rnd.to_string());
        }
        if result.contains("{{$isoTimestamp}}") {
            result = result.replace("{{$isoTimestamp}}", &chrono::Utc::now().to_rfc3339());
        }

        // 2. Variáveis do usuário
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

impl VariableResolverPort for RealVariableResolver {
    fn resolve(
        &self,
        text: &str,
        env_vars: &HashMap<String, String>,
        collection_vars: &HashMap<String, String>,
        global_vars: &HashMap<String, String>,
        session_vars: &HashMap<String, String>,
    ) -> Result<String, DomainError> {
        // Prioridade (da menor para a maior): Global < Collection < Environment < Session
        let mut merged = global_vars.clone();
        merged.extend(collection_vars.clone());
        merged.extend(env_vars.clone());
        merged.extend(session_vars.clone());

        Ok(Self::substitute(text, &merged))
    }
}
