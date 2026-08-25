use crate::domain::models::{CollectionItem, Environment, CollectionRunReport, RequestRunResult};
use crate::application::commands::send_request::SendRequestUseCase;
use crate::domain::errors::DomainError;

pub struct RunCollectionUseCase {
    send_request_usecase: SendRequestUseCase,
}

impl RunCollectionUseCase {
    pub fn new(send_request_usecase: SendRequestUseCase) -> Self {
        Self { send_request_usecase }
    }

    pub async fn execute(&self, items: Vec<CollectionItem>, environment: &Environment) -> Result<CollectionRunReport, DomainError> {
        let mut report = CollectionRunReport {
            total_requests: 0,
            total_tests: 0,
            passed_tests: 0,
            results: vec![],
        };

        let mut current_env = environment.clone();

        for item in items {
            self.run_item(item, &mut current_env, &mut report).await?;
        }

        Ok(report)
    }

    // Usando recursão asíncrona com Box::pin
    async fn run_item(&self, item: CollectionItem, env: &mut Environment, report: &mut CollectionRunReport) -> Result<(), DomainError> {
        match item {
            CollectionItem::Request(req) => {
                let (resp, updated_env) = self.send_request_usecase.execute(req.clone(), env).await?;
                *env = updated_env;
                
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
                    // Evita recursão infinita e satisfaz o compilador para async recursivo
                    let fut = self.run_item(sub_item, env, report);
                    Box::pin(fut).await?;
                }
            }
        }
        Ok(())
    }
}
