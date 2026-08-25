use crate::domain::models::{HttpRequest, LoadTestConfig, LoadTestReport};
use crate::application::ports::http_client::HttpClientPort;
use crate::application::ports::variable_resolver::VariableResolverPort;
use crate::application::services::load_test_service::{
    LatestProgressSlot, LoadTestRun, LoadTestService,
};
use crate::domain::errors::{AppError, DomainError};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::time::Instant;

// --- Streaming Tokio load testing engine (P4 epic) ---------------------------
//
// Thin IPC adapter around `LoadTestService`. The service owns the worker
// pool and the single-writer aggregator; this layer only wires Tauri state,
// spawns the background orchestration task and forwards throttled progress
// snapshots to the Webview through the `load_test_progress` event.

use crate::domain::models::{
    Environment, GlobalVariables, LoadTestConfigDto, LoadTestProgressEventDto,
};
use tokio::sync::watch;
use tauri::Emitter;

/// Event channel name streamed to the Webview every sampling window.
pub const LOAD_TEST_PROGRESS_EVENT: &str = "load_test_progress";

/// Managed handle to the currently running (or last finished) load test.
pub struct LoadTestState {
    cancellation: Arc<watch::Sender<bool>>,
    latest_progress: LatestProgressSlot,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl LoadTestState {
    pub fn new() -> Self {
        let (cancellation, _) = watch::channel(false);
        Self {
            cancellation: Arc::new(cancellation),
            latest_progress: Arc::new(std::sync::Mutex::new(None)),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl Default for LoadTestState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn start_load_test(
    config: LoadTestConfigDto,
    environment: Environment,
    globals: GlobalVariables,
    service: tauri::State<'_, Arc<LoadTestService>>,
    state: tauri::State<'_, LoadTestState>,
    app_handle: tauri::AppHandle,
) -> Result<String, AppError> {
    if state
        .is_running
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return Err(AppError {
            code: "VALIDATION_ERROR".to_string(),
            message: "A load test is already running".to_string(),
        });
    }

    let test_id = uuid::Uuid::new_v4().to_string();
    // Reset the shared cancellation flag for this fresh run.
    let _ = state.cancellation.send_replace(false);
    let cancel_rx = state.cancellation.subscribe();
    let latest_slot = Arc::clone(&state.latest_progress);
    let is_running = Arc::clone(&state.is_running);

    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::channel::<LoadTestProgressEventDto>(64);
    let run_test_id = test_id.clone();
    let runner_service = Arc::clone(&*service);

    tauri::async_runtime::spawn(async move {
        // Drive the engine and the event-forwarding loop concurrently: the
        // forward loop ends when the aggregator drops its sender, which is
        // exactly when the terminal `is_finished` snapshot was delivered.
        let run = LoadTestRun {
            config,
            environment: Arc::new(environment),
            globals: Arc::new(globals),
            test_id: run_test_id,
            cancel_rx,
            progress_tx,
            latest_slot: latest_slot.clone(),
        };
        let run_future = runner_service.run(run);
        let forward_future = async {
            while let Some(event) = progress_rx.recv().await {
                if let Ok(mut slot) = latest_slot.lock() {
                    *slot = Some(event.clone());
                }
                let _ = app_handle.emit(LOAD_TEST_PROGRESS_EVENT, event);
            }
        };
        let (run_result, ()) = tokio::join!(run_future, forward_future);
        if let Err(error) = run_result {
            eprintln!("load test failed: {error}");
        }
        is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    Ok(test_id)
}

#[tauri::command]
pub async fn stop_load_test(
    state: tauri::State<'_, LoadTestState>,
) -> Result<Option<LoadTestProgressEventDto>, AppError> {
    let _ = state.cancellation.send_replace(true);
    Ok(state
        .latest_progress
        .lock()
        .ok()
        .and_then(|slot| slot.clone()))
}

#[tauri::command]
pub async fn get_load_test_status(
    state: tauri::State<'_, LoadTestState>,
) -> Result<Option<LoadTestProgressEventDto>, AppError> {
    Ok(state
        .latest_progress
        .lock()
        .ok()
        .and_then(|slot| slot.clone()))
}

// --- Legacy sequential load test use case ------------------------------------
//
// Retained unchanged for the blocking `run_load_test` IPC contract used by
// older clients. New consumers must prefer `start_load_test` above, which
// streams sampled metrics without mutex contention on the hot path.

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
