use crate::domain::models::{HttpRequest, LoadTestConfig, LoadTestReport};
use crate::application::ports::http_client::HttpClientPort;
use crate::application::ports::variable_resolver::VariableResolverPort;
use crate::domain::errors::DomainError;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::time::Instant;

const MAX_CONCURRENT_REQUESTS: usize = 100;

pub struct LoadTestUseCase {
    http_client: Arc<dyn HttpClientPort>,
    variable_resolver: Arc<dyn VariableResolverPort>,
}

impl LoadTestUseCase {
    pub fn new(
        http_client: Arc<dyn HttpClientPort>,
        variable_resolver: Arc<dyn VariableResolverPort>,
    ) -> Self {
        Self { http_client, variable_resolver }
    }

    pub async fn execute(
        &self,
        request: HttpRequest,
        config: LoadTestConfig,
        environment: crate::domain::models::Environment,
        globals: crate::domain::models::GlobalVariables,
    ) -> Result<LoadTestReport, DomainError> {
        let total_planned = config.users * config.requests_per_user;
        let success_count = Arc::new(Mutex::new(0u32));
        let failure_count = Arc::new(Mutex::new(0u32));
        let all_times = Arc::new(Mutex::new(Vec::with_capacity(total_planned as usize)));

        // Semaphore to limit concurrent requests and prevent file descriptor exhaustion
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

        let start_test = Instant::now();
        let mut handlers = vec![];

        for _ in 0..config.users {
            let client = Arc::clone(&self.http_client);
            let resolver = Arc::clone(&self.variable_resolver);
            let req = request.clone();
            let env = environment.clone();
            let globs = globals.clone();
            let sess = std::collections::HashMap::new();
            let succ = Arc::clone(&success_count);
            let fail = Arc::clone(&failure_count);
            let tms = Arc::clone(&all_times);
            let req_count = config.requests_per_user;
            let delay = config.delay_ms;
            let sem = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                for _ in 0..req_count {
                    // Acquire semaphore permit to limit concurrency
                    let _permit = sem.acquire().await;

                    let mut active_req = req.clone();
                    let active_env = env.clone();
                    let active_globs = globs.clone();

                    let start = Instant::now();

                    // Variable resolution
                    let collection_vars = &active_req.variables;

                    let resolved_url = resolver.resolve(
                        &active_req.url.0,
                        &active_env.to_runtime_map(),
                        collection_vars,
                        &active_globs.variables,
                        &sess
                    );

                    if let Ok(url_str) = resolved_url {
                        active_req.url.0 = url_str;

                        let result = client.send(active_req).await;
                        let duration = start.elapsed().as_millis() as u64;

                        let mut t_guard = tms.lock().await;
                        t_guard.push(duration);

                        match result {
                            Ok(_) => {
                                let mut s_guard = succ.lock().await;
                                *s_guard += 1;
                            },
                            Err(_) => {
                                let mut f_guard = fail.lock().await;
                                *f_guard += 1;
                            }
                        }
                    } else {
                        let mut f_guard = fail.lock().await;
                        *f_guard += 1;
                    }

                    // Permit is automatically released when _permit goes out of scope

                    if delay > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                }
            });
            handlers.push(handle);
        }

        for h in handlers {
            let _ = h.await;
        }

        let total_duration = start_test.elapsed();
        let final_times = all_times.lock().await;
        let final_success = *success_count.lock().await;
        let final_failure = *failure_count.lock().await;

        if final_times.is_empty() {
            return Err(DomainError::NetworkError("No requests were completed".into()));
        }

        let mut sorted_times = final_times.clone();
        sorted_times.sort();

        let sum: u64 = sorted_times.iter().sum();
        let avg = sum as f64 / sorted_times.len() as f64;
        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];
        let p95_idx = (sorted_times.len() as f64 * 0.95) as usize;
        let p95 = sorted_times[p95_idx.min(sorted_times.len() - 1)];

        let rps = (final_success + final_failure) as f64 / total_duration.as_secs_f64();

        Ok(LoadTestReport {
            total_requests: final_success + final_failure,
            success_count: final_success,
            failure_count: final_failure,
            avg_time_ms: avg,
            min_time_ms: min,
            max_time_ms: max,
            p95_time_ms: p95,
            requests_per_second: rps,
        })
    }
}
