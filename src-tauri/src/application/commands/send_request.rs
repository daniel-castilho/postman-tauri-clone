// src-tauri/src/application/commands/send_request.rs
use crate::domain::models::{HttpRequest, HttpResponse};
use crate::domain::errors::DomainError;
use crate::application::ports::{
    http_client::HttpClientPort,
    grpc_client::GrpcClientPort,
    variable_resolver::VariableResolverPort,
    script_runner::ScriptRunnerPort,
};

use std::sync::Arc;

#[derive(Clone)]
pub struct SendRequestUseCase {
    http_client: Arc<dyn HttpClientPort>,
    grpc_client: Arc<dyn GrpcClientPort>,
    variable_resolver: Arc<dyn VariableResolverPort>,
    script_runner: Arc<dyn ScriptRunnerPort>,
}

impl SendRequestUseCase {
    pub fn new(
        http_client: Arc<dyn HttpClientPort>,
        grpc_client: Arc<dyn GrpcClientPort>,
        variable_resolver: Arc<dyn VariableResolverPort>,
        script_runner: Arc<dyn ScriptRunnerPort>,
    ) -> Self {
        Self { http_client, grpc_client, variable_resolver, script_runner }
    }

    pub async fn execute(
        &self, 
        mut request: HttpRequest, 
        environment: &crate::domain::models::Environment,
        globals: &crate::domain::models::GlobalVariables,
        session_vars: &std::collections::HashMap<String, String>,
    ) -> Result<(HttpResponse, crate::domain::models::Environment, crate::domain::models::GlobalVariables, std::collections::HashMap<String, String>), DomainError> {
        let mut active_env = environment.clone();
        let mut active_globals = globals.clone();
        let mut active_session = session_vars.clone();
        let mut all_logs = vec![];

        // 1. Executa Pre-request Script (se existir)
        let pre_request = request.scripts.as_ref().map(|s| s.pre_request.clone()).unwrap_or_default();
        if !pre_request.is_empty() {
            let logs = self.script_runner.execute_pre_request(
                &pre_request, 
                &mut request, 
                &mut active_env.variables, 
                &mut active_globals.variables,
                &mut active_session
            )?;
            all_logs.extend(logs);
        }

        // 2. Resolve variáveis
        self.resolve_variables(&mut request, &active_env.variables, &active_globals.variables, &active_session)?;

        // 3. Envia a request
        let mut response = if matches!(request.method, crate::domain::models::HttpMethod::GRPC) {
            self.grpc_client.call(request.clone()).await?
        } else {
            self.http_client.send(request.clone()).await?
        };

        // 4. Executa Test Script (se existir)
        let test_script = request.scripts.as_ref().map(|s| s.tests.clone()).unwrap_or_default();
        if !test_script.is_empty() {
            let (test_results, test_logs) = self.script_runner.execute_test(
                &test_script, 
                &response, 
                &mut active_env.variables,
                &mut active_globals.variables,
                &mut active_session
            )?;
            response.tests_results = test_results;
            all_logs.extend(test_logs);
        }

        response.logs = all_logs;

        Ok((response, active_env, active_globals, active_session))
    }

    fn resolve_variables(
        &self,
        request: &mut HttpRequest,
        env_vars: &std::collections::HashMap<String, String>,
        global_vars: &std::collections::HashMap<String, String>,
        session_vars: &std::collections::HashMap<String, String>,
    ) -> Result<(), DomainError> {
        // Resolve URL
        let collection_vars = &request.variables;
        request.url = crate::domain::models::Url(
            self.variable_resolver.resolve(&request.url.0, env_vars, collection_vars, global_vars, session_vars)?
        );

        // Resolve Headers
        for header in request.headers.iter_mut() {
            if header.enabled {
                header.value = self.variable_resolver.resolve(&header.value, env_vars, collection_vars, global_vars, session_vars)?;
            }
        }

        // Resolve Body (somente para Raw por enquanto)
        if let Some(crate::domain::models::Body::Raw(ref mut body_str, _)) = request.body {
            *body_str = self.variable_resolver.resolve(body_str, env_vars, collection_vars, global_vars, session_vars)?;
        }

        Ok(())
    }
}
