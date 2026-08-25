// src-tauri/src/application/ports/script_runner.rs
use crate::domain::errors::DomainError;
use crate::domain::models::{HttpRequest, HttpResponse};

use std::collections::HashMap;

pub trait ScriptRunnerPort: Send + Sync {
    fn execute_pre_request(
        &self, 
        script: &str, 
        request: &mut HttpRequest, 
        env_vars: &mut HashMap<String, String>,
        global_vars: &mut HashMap<String, String>,
        session_vars: &mut HashMap<String, String>
    ) -> Result<Vec<crate::domain::models::ScriptLog>, DomainError>;

    fn execute_test(
        &self, 
        script: &str, 
        response: &HttpResponse, 
        env_vars: &mut HashMap<String, String>,
        global_vars: &mut HashMap<String, String>,
        session_vars: &mut HashMap<String, String>
    ) -> Result<(Vec<crate::domain::models::TestResult>, Vec<crate::domain::models::ScriptLog>), DomainError>;
}
