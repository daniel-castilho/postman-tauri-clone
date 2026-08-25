use crate::domain::models::{CollectionItem, Environment, GlobalVariables, CollectionRunReport, RequestRunResult};
use crate::application::commands::send_request::SendRequestUseCase;
use crate::domain::errors::DomainError;

use std::collections::HashMap;

pub struct RunCollectionUseCase {
    send_request_usecase: SendRequestUseCase,
}

impl RunCollectionUseCase {
    pub fn new(send_request_usecase: SendRequestUseCase) -> Self {
        Self { send_request_usecase }
    }

    pub async fn execute(
        &self,
        items: Vec<CollectionItem>,
        environment: &Environment,
        globals: &GlobalVariables,
        session_vars: &HashMap<String, String>,
    ) -> Result<CollectionRunReport, DomainError> {
        let mut report = CollectionRunReport {
            total_requests: 0,
            total_tests: 0,
            passed_tests: 0,
            results: vec![],
        };

        let mut current_env = environment.clone();
        let mut current_globals = globals.clone();
        let mut current_session = session_vars.clone();

        for item in items {
            self.run_item(item, &mut current_env, &mut current_globals, &mut current_session, &mut report).await?;
        }

        Ok(report)
    }

    // Async recursion via Box::pin
    async fn run_item(
        &self,
        item: CollectionItem,
        env: &mut Environment,
        globals: &mut GlobalVariables,
        session_vars: &mut HashMap<String, String>,
        report: &mut CollectionRunReport,
    ) -> Result<(), DomainError> {
        match item {
            CollectionItem::Request(req) => {
                let req = *req;
                let (resp, updated_env, updated_globals, updated_session) = self
                    .send_request_usecase
                    .execute(req.clone(), env, globals, session_vars)
                    .await?;
                *env = updated_env;
                *globals = updated_globals;
                *session_vars = updated_session;
                
                let test_count = resp.tests_results.len();
                let passed_count = resp.tests_results.iter().filter(|t| t.passed).count();
                
                report.total_requests += 1;
                report.total_tests += test_count;
                report.passed_tests += passed_count;
                
                report.results.push(RequestRunResult {
                    request_name: req.name.clone(),
                    status: resp.status,
                    time_ms: resp.time_ms,
                    tests: resp.tests_results,
                });
            },
            CollectionItem::Folder { items, .. } => {
                for sub_item in items {
                    // Avoids infinite recursion and satisfies the compiler for async recursion
                    let fut = self.run_item(sub_item, env, globals, session_vars, report);
                    Box::pin(fut).await?;
                }
            }
        }
        Ok(())
    }
}
