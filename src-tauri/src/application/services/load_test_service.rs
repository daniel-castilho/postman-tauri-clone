// src-tauri/src/application/services/load_test_service.rs
//
// Tokio load testing engine (P4 epic).
//
// Design rules enforced here:
// - Single-writer aggregation: every worker pushes a tiny `Sample` into an
//   `mpsc` channel; ONLY the aggregator task touches accumulated metrics.
//   No mutex is ever acquired inside the request hot loop.
// - Throttled streaming: the aggregator computes one metrics snapshot per
//   sampling tick (200ms default) so the IPC bridge never sees floods of
//   per-request events.
// - Graceful cancellation: a `tokio::sync::watch` flag is selected upon by
//   every worker and by the aggregator, halting execution immediately.
//
// Percentiles are calculated over sorted sample windows (no external
// histogram dependency required for the P4 scope).

use crate::application::ports::http_client::HttpClientPort;
use crate::application::ports::variable_resolver::VariableResolverPort;
use crate::domain::errors::DomainError;
use crate::domain::models::{
    Environment, GlobalVariables, HttpRequest, LatencyPercentilesDto, LoadTestConfigDto,
    LoadTestProgressEventDto, StatusCodeCountDto,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

/// Default sampling window between two streamed progress events.
pub const SAMPLING_INTERVAL_MS: u64 = 200;
/// Bounded sample queue between workers and the single-writer aggregator.
const SAMPLE_CHANNEL_CAPACITY: usize = 10_000;
/// Number of sampling windows kept for the smoothed RPS/throughput average.
const RPS_SMOOTHING_WINDOWS: usize = 10;

pub const MIN_VIRTUAL_USERS: u32 = 1;
pub const MAX_VIRTUAL_USERS: u32 = 500;
pub const MIN_DURATION_SECONDS: u64 = 1;
pub const MAX_DURATION_SECONDS: u64 = 3600;

/// One finished HTTP iteration produced by a worker task.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub duration_ms: u32,
    /// HTTP status code, or `0` for transport errors and timeouts.
    pub status_code: u16,
    pub bytes: usize,
}

/// Latest metrics snapshot slot shared with the IPC layer for status polling.
pub type LatestProgressSlot = Arc<Mutex<Option<LoadTestProgressEventDto>>>;

/// Multi-threaded Tokio load testing engine.
pub struct LoadTestService {
    http_client: Arc<dyn HttpClientPort>,
    variable_resolver: Arc<dyn VariableResolverPort>,
}

/// Everything the engine needs to execute one load test run.
pub struct LoadTestRun {
    pub config: LoadTestConfigDto,
    pub environment: Arc<Environment>,
    pub globals: Arc<GlobalVariables>,
    pub test_id: String,
    pub cancel_rx: watch::Receiver<bool>,
    pub progress_tx: mpsc::Sender<LoadTestProgressEventDto>,
    pub latest_slot: LatestProgressSlot,
}

/// Immutable per-run parameters shared by every spawned worker task.
struct WorkerParams {
    virtual_users: u32,
    ramp_up_seconds: u64,
    duration_seconds: u64,
    timeout_ms: u64,
    request: HttpRequest,
    cancel_rx: watch::Receiver<bool>,
    sample_tx: mpsc::Sender<Sample>,
}

impl LoadTestService {
    pub fn new(
        http_client: Arc<dyn HttpClientPort>,
        variable_resolver: Arc<dyn VariableResolverPort>,
    ) -> Self {
        Self {
            http_client,
            variable_resolver,
        }
    }

    /// Runs a full load test until `duration_seconds` elapses, the target is
    /// reached, or cancellation is signalled through the cancellation watch.
    ///
    /// Progress snapshots are streamed through `run.progress_tx`; the returned
    /// event is the final snapshot carrying `is_finished: true`.
    pub async fn run(&self, run: LoadTestRun) -> Result<LoadTestProgressEventDto, DomainError> {
        let LoadTestRun {
            config,
            environment,
            globals,
            test_id,
            cancel_rx,
            progress_tx,
            latest_slot,
        } = run;
        validate_config(&config)?;
        let resolved_request = self.resolve_target(&config.target_request, &environment, &globals)?;

        let started_at = Instant::now();
        let (sample_tx, sample_rx) = mpsc::channel::<Sample>(SAMPLE_CHANNEL_CAPACITY);

        let worker_params = WorkerParams {
            virtual_users: config.virtual_users,
            ramp_up_seconds: config.ramp_up_seconds,
            duration_seconds: config.duration_seconds,
            timeout_ms: config.timeout_ms,
            request: resolved_request,
            cancel_rx: cancel_rx.clone(),
            sample_tx,
        };

        // Worker pool: one lightweight Tokio task per virtual user.
        let mut worker_handles = Vec::with_capacity(config.virtual_users as usize);
        for worker_index in 0..config.virtual_users {
            worker_handles.push(self.spawn_worker(worker_index, &worker_params));
        }
        // Workers hold clones; the aggregator terminates once this original
        // sender AND every worker clone are dropped.
        drop(worker_params.sample_tx);

        let aggregator = tokio::spawn(aggregator_task(AggregatorParams {
            sample_rx,
            progress_tx,
            cancel_rx,
            started_at,
            planned_vus: config.virtual_users,
            duration_seconds: config.duration_seconds,
            test_id,
            latest_slot,
        }));

        for handle in worker_handles {
            let _ = handle.await;
        }

        aggregator
            .await
            .map_err(|error| DomainError::NetworkError(format!("load test aggregator failed: {error}")))
    }

    fn spawn_worker(
        &self,
        worker_index: u32,
        params: &WorkerParams,
    ) -> tokio::task::JoinHandle<()> {
        let WorkerParams {
            virtual_users,
            ramp_up_seconds,
            duration_seconds,
            timeout_ms,
            request,
            cancel_rx,
            sample_tx,
        } = params;
        let http_client = Arc::clone(&self.http_client);
        let virtual_users = *virtual_users;
        let ramp_up_seconds = *ramp_up_seconds;
        let duration_seconds = *duration_seconds;
        let timeout_ms = *timeout_ms;
        let request = request.clone();
        let cancel_rx = cancel_rx.clone();
        let sample_tx = sample_tx.clone();

        tokio::spawn(async move {
            // Ramp-up: stagger worker start times across the ramp-up window.
            let start_delay = if virtual_users > 1 {
                ramp_up_seconds * u64::from(worker_index) / u64::from(virtual_users - 1).max(1)
            } else {
                0
            };
            if !sleep_with_cancel(start_delay, &cancel_rx).await {
                return;
            }

            let deadline = Instant::now() + Duration::from_secs(duration_seconds);
            let timeout = Duration::from_millis(timeout_ms.max(1));

            loop {
                if *cancel_rx.borrow() || Instant::now() >= deadline {
                    break;
                }

                let request_started = Instant::now();
                let mut cancel_on_request = cancel_rx.clone();
                let outcome = tokio::select! {
                    changed = cancel_on_request.changed() => {
                        // Cancelled while the request was in flight.
                        let _ = changed;
                        break;
                    }
                    result = tokio::time::timeout(timeout, http_client.send(request.clone())) => {
                        match result {
                            Ok(Ok(response)) => Sample {
                                duration_ms: request_started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
                                status_code: response.status,
                                bytes: response.size_bytes,
                            },
                            _ => Sample {
                                duration_ms: request_started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
                                status_code: 0,
                                bytes: 0,
                            },
                        }
                    }
                };

                // A full channel back-pressure means the aggregator is gone;
                // stop instead of spinning forever.
                if sample_tx.send(outcome).await.is_err() {
                    break;
                }
            }
        })
    }

    /// Resolves environment/collection/global variables in the target URL
    /// once per run (the variable context is static during a load test).
    fn resolve_target(
        &self,
        request: &HttpRequest,
        environment: &Environment,
        globals: &GlobalVariables,
    ) -> Result<HttpRequest, DomainError> {
        let session_vars = HashMap::new();
        let resolved_url = self.variable_resolver.resolve(
            &request.url.0,
            &environment.to_runtime_map(),
            &request.variables,
            &globals.variables,
            &session_vars,
        )?;

        let mut resolved = request.clone();
        resolved.url.0 = resolved_url;
        Ok(resolved)
    }
}

struct AggregatorParams {
    sample_rx: mpsc::Receiver<Sample>,
    progress_tx: mpsc::Sender<LoadTestProgressEventDto>,
    cancel_rx: watch::Receiver<bool>,
    started_at: Instant,
    planned_vus: u32,
    duration_seconds: u64,
    test_id: String,
    latest_slot: LatestProgressSlot,
}

/// Single-writer task: owns ALL accumulated metrics, emits throttled
/// snapshots and the terminal event.
async fn aggregator_task(params: AggregatorParams) -> LoadTestProgressEventDto {
    let AggregatorParams {
        mut sample_rx,
        progress_tx,
        mut cancel_rx,
        started_at,
        planned_vus,
        duration_seconds,
        test_id,
        latest_slot,
    } = params;

    let mut metrics = Metrics::default();
    let mut window_counts: Vec<u64> = Vec::with_capacity(RPS_SMOOTHING_WINDOWS);
    let mut window_bytes: Vec<u64> = Vec::with_capacity(RPS_SMOOTHING_WINDOWS);
    let mut last_tick_at = started_at;
    let mut last_tick_total = 0u64;
    let mut last_tick_bytes = 0u64;
    let mut tick = tokio::time::interval(Duration::from_millis(SAMPLING_INTERVAL_MS));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut cancelled = false;

    loop {
        tokio::select! {
            maybe_sample = sample_rx.recv() => {
                match maybe_sample {
                    Some(sample) => metrics.record(sample),
                    // All workers exited and every sample was consumed.
                    None => break,
                }
            }
            _ = tick.tick() => {
                if cancelled {
                    continue;
                }
                let event = build_snapshot(
                    &metrics,
                    &mut window_counts,
                    &mut window_bytes,
                    &mut last_tick_total,
                    &mut last_tick_bytes,
                    &mut last_tick_at,
                    started_at,
                    planned_vus,
                    duration_seconds,
                    &test_id,
                    false,
                );
                if let Ok(mut slot) = latest_slot.lock() {
                    *slot = Some(event.clone());
                }
                let _ = progress_tx.send(event).await;
            }
            changed = cancel_rx.changed() => {
                if changed.is_err() || *cancel_rx.borrow() {
                    // Cancelled, or watch sender dropped (app teardown):
                    // stop ticking and drain remaining samples until done.
                    cancelled = true;
                }
            }
        }
    }

    // Terminal snapshot with the complete distribution.
    let final_event = build_snapshot(
        &metrics,
        &mut window_counts,
        &mut window_bytes,
        &mut last_tick_total,
        &mut last_tick_bytes,
        &mut last_tick_at,
        started_at,
        planned_vus,
        duration_seconds,
        &test_id,
        true,
    );
    if let Ok(mut slot) = latest_slot.lock() {
        *slot = Some(final_event.clone());
    }
    let _ = progress_tx.send(final_event.clone()).await;
    final_event
}

/// ALL accumulated run metrics. Only the aggregator task ever touches this.
#[derive(Default)]
struct Metrics {
    latencies_ms: Vec<u32>,
    status_codes: HashMap<u16, u64>,
    successful: u64,
    failed: u64,
    total_bytes: u64,
}

impl Metrics {
    fn record(&mut self, sample: Sample) {
        self.latencies_ms.push(sample.duration_ms);
        *self.status_codes.entry(sample.status_code).or_insert(0) += 1;
        if is_successful(sample.status_code) {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        self.total_bytes += sample.bytes as u64;
    }

    fn latencies(&self) -> &[u32] {
        &self.latencies_ms
    }

    fn status_codes(&self) -> &HashMap<u16, u64> {
        &self.status_codes
    }

    fn successful(&self) -> u64 {
        self.successful
    }

    fn failed(&self) -> u64 {
        self.failed
    }

    fn bytes(&self) -> u64 {
        self.total_bytes
    }
}

fn is_successful(status_code: u16) -> bool {
    (200..400).contains(&status_code)
}

#[allow(clippy::too_many_arguments)]
fn build_snapshot(
    metrics: &Metrics,
    window_counts: &mut Vec<u64>,
    window_bytes: &mut Vec<u64>,
    last_tick_total: &mut u64,
    last_tick_bytes: &mut u64,
    last_tick_at: &mut Instant,
    started_at: Instant,
    planned_vus: u32,
    duration_seconds: u64,
    test_id: &str,
    is_finished: bool,
) -> LoadTestProgressEventDto {
    let now = Instant::now();
    let total = metrics.successful() + metrics.failed();
    let window_elapsed = now
        .duration_since(*last_tick_at)
        .as_secs_f64()
        .max(0.001);
    let window_rps = (total - *last_tick_total) as f64 / window_elapsed;

    window_counts.push(total - *last_tick_total);
    window_bytes.push(metrics.bytes() - *last_tick_bytes);
    if window_counts.len() > RPS_SMOOTHING_WINDOWS {
        window_counts.remove(0);
        window_bytes.remove(0);
    }
    let smoothed_windows = window_counts.iter().sum::<u64>().max(1) as f64
        / (window_counts.len() as f64 * SAMPLING_INTERVAL_MS as f64 / 1000.0);
    let smoothed_bps = window_bytes.iter().sum::<u64>() as f64
        / (window_bytes.len() as f64 * SAMPLING_INTERVAL_MS as f64 / 1000.0);

    *last_tick_total = total;
    *last_tick_bytes = metrics.bytes();
    *last_tick_at = now;

    let elapsed = now.duration_since(started_at).as_secs_f64();
    let capped_elapsed = if is_finished {
        elapsed
    } else {
        elapsed.min(f64::from(u32::MAX))
    };

    let active_vus = if is_finished {
        0
    } else {
        estimate_active_vus(planned_vus, duration_seconds, elapsed)
    };

    LoadTestProgressEventDto {
        test_id: test_id.to_string(),
        elapsed_seconds: capped_elapsed,
        active_vus,
        current_rps: smoothed_rps(smoothed_windows, window_rps),
        total_requests: total,
        successful_requests: metrics.successful(),
        failed_requests: metrics.failed(),
        bytes_per_second: if window_bytes.iter().sum::<u64>() == 0 {
            0.0
        } else {
            smoothed_bps
        },
        percentiles: compute_percentiles(metrics.latencies()),
        status_codes: metrics
            .status_codes()
            .iter()
            .map(|(code, count)| StatusCodeCountDto {
                code: *code,
                count: *count,
            })
            .collect(),
        is_finished,
    }
}

/// Smoothing keeps the chart readable while staying responsive: blend the
/// windowed instantaneous rate with the short moving average.
fn smoothed_rps(moving_average: f64, instant: f64) -> f64 {
    (moving_average * 0.7 + instant * 0.3).max(0.0)
}

fn estimate_active_vus(planned_vus: u32, ramp_up_seconds: u64, elapsed_seconds: f64) -> u32 {
    let ramp_up = ramp_up_seconds.min(u64::from(u32::MAX));
    if ramp_up == 0 || elapsed_seconds >= f64::from(u32::try_from(ramp_up).unwrap_or(u32::MAX)) {
        return planned_vus;
    }
    let ratio = elapsed_seconds / f64::from(u32::try_from(ramp_up).unwrap_or(u32::MAX));
    ((planned_vus as f64 * ratio).ceil() as u32).clamp(1, planned_vus)
}

/// Accurate nearest-rank percentiles over the sorted latency sample window.
pub fn compute_percentiles(latencies_ms: &[u32]) -> LatencyPercentilesDto {
    if latencies_ms.is_empty() {
        return LatencyPercentilesDto {
            p50_ms: 0.0,
            p90_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
        };
    }

    let mut sorted = latencies_ms.to_vec();
    sorted.sort_unstable();
    let pick = |percentile: f64| -> f64 {
        let index = ((sorted.len() as f64 * percentile).ceil() as usize)
            .saturating_sub(1)
            .min(sorted.len() - 1);
        f64::from(sorted[index])
    };

    let sum: u64 = sorted.iter().map(|value| u64::from(*value)).sum();
    LatencyPercentilesDto {
        p50_ms: pick(0.50),
        p90_ms: pick(0.90),
        p95_ms: pick(0.95),
        p99_ms: pick(0.99),
        min_ms: f64::from(sorted[0]),
        max_ms: f64::from(sorted[sorted.len() - 1]),
        mean_ms: sum as f64 / sorted.len() as f64,
    }
}

pub fn validate_config(config: &LoadTestConfigDto) -> Result<(), DomainError> {
    if !(MIN_VIRTUAL_USERS..=MAX_VIRTUAL_USERS).contains(&config.virtual_users) {
        return Err(DomainError::ValidationError(format!(
            "virtual_users must be between {MIN_VIRTUAL_USERS} and {MAX_VIRTUAL_USERS}"
        )));
    }
    if !(MIN_DURATION_SECONDS..=MAX_DURATION_SECONDS).contains(&config.duration_seconds) {
        return Err(DomainError::ValidationError(format!(
            "duration_seconds must be between {MIN_DURATION_SECONDS} and {MAX_DURATION_SECONDS}"
        )));
    }
    if config.ramp_up_seconds > config.duration_seconds {
        return Err(DomainError::ValidationError(
            "ramp_up_seconds cannot exceed duration_seconds".to_string(),
        ));
    }
    if config.timeout_ms == 0 {
        return Err(DomainError::ValidationError(
            "timeout_ms must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

/// Sleeps for `seconds`, returning `false` when cancellation arrives first.
async fn sleep_with_cancel(seconds: u64, cancel_rx: &watch::Receiver<bool>) -> bool {
    if seconds == 0 {
        return !*cancel_rx.borrow();
    }
    let mut rx = cancel_rx.clone();
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => true,
        changed = rx.changed() => changed.is_err() || *rx.borrow(),
    }
}

// --- Unit tests -------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{BodyMode, Header, HttpMethod, RequestId, Url};

    struct NoopResolver;

    impl VariableResolverPort for NoopResolver {
        fn resolve(
            &self,
            text: &str,
            _env: &HashMap<String, String>,
            _collection: &HashMap<String, String>,
            _globals: &HashMap<String, String>,
            _session: &HashMap<String, String>,
        ) -> Result<String, DomainError> {
            Ok(text.to_string())
        }
    }

    struct FixedHttpClient {
        status: u16,
        body_size: usize,
    }

    #[async_trait::async_trait]
    impl HttpClientPort for FixedHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<crate::domain::models::HttpResponse, DomainError> {
            Ok(crate::domain::models::HttpResponse {
                status: self.status,
                status_text: "OK".to_string(),
                headers: Vec::<Header>::new(),
                body: Some("x".repeat(self.body_size)),
                time_ms: 1,
                size_bytes: self.body_size,
                tests_results: Vec::new(),
                logs: Vec::new(),
            })
        }
    }

    fn sample_config(duration_seconds: u64) -> LoadTestConfigDto {
        LoadTestConfigDto {
            target_request: HttpRequest {
                id: RequestId("load-test-target".to_string()),
                name: "Target".to_string(),
                description: None,
                method: HttpMethod::GET,
                url: Url("http://127.0.0.1:1/ping".to_string()),
                headers: Vec::new(),
                body: Some(crate::domain::models::Body::Raw("{}".to_string(), BodyMode::Json)),
                auth: None,
                variables: HashMap::new(),
                scripts: None,
                grpc_config: None,
            },
            virtual_users: 4,
            duration_seconds,
            ramp_up_seconds: 0,
            timeout_ms: 500,
        }
    }

    fn empty_context() -> (Arc<Environment>, Arc<GlobalVariables>) {
        (
            Arc::new(Environment {
                id: "env".to_string(),
                name: "test".to_string(),
                variables: Vec::new(),
            }),
            Arc::new(GlobalVariables {
                variables: HashMap::new(),
            }),
        )
    }

    #[test]
    fn percentile_calculator_matches_expected_values() {
        let latencies: Vec<u32> = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let percentiles = compute_percentiles(&latencies);
        assert_eq!(percentiles.min_ms, 10.0);
        assert_eq!(percentiles.max_ms, 100.0);
        assert_eq!(percentiles.mean_ms, 55.0);
        assert_eq!(percentiles.p50_ms, 50.0);
        assert_eq!(percentiles.p90_ms, 90.0);
        assert_eq!(percentiles.p95_ms, 100.0);
        assert_eq!(percentiles.p99_ms, 100.0);
    }

    #[test]
    fn percentile_calculator_handles_empty_samples() {
        let percentiles = compute_percentiles(&[]);
        assert_eq!(percentiles.p50_ms, 0.0);
        assert_eq!(percentiles.mean_ms, 0.0);
    }

    #[test]
    fn validate_config_rejects_out_of_range_values() {
        let mut config = sample_config(5);
        assert!(validate_config(&config).is_ok());

        config.virtual_users = 501;
        assert!(validate_config(&config).is_err());

        config.virtual_users = 0;
        assert!(validate_config(&config).is_err());

        config.virtual_users = 8;
        config.duration_seconds = 3601;
        assert!(validate_config(&config).is_err());

        config.duration_seconds = 4;
        config.ramp_up_seconds = 5;
        assert!(validate_config(&config).is_err());
    }

    #[tokio::test]
    async fn engine_streams_progress_and_finishes_cleanly() {
        let service = LoadTestService::new(
            Arc::new(FixedHttpClient {
                status: 200,
                body_size: 128,
            }),
            Arc::new(NoopResolver),
        );
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (progress_tx, mut progress_rx) = mpsc::channel::<LoadTestProgressEventDto>(256);
        let latest_slot: LatestProgressSlot = Arc::new(Mutex::new(None));

        let (environment, globals) = empty_context();
        let run = LoadTestRun {
            config: sample_config(1),
            environment,
            globals,
            test_id: "test-stream".to_string(),
            cancel_rx,
            progress_tx,
            latest_slot,
        };
        let runner = tokio::spawn(async move { service.run(run).await });

        let mut events = 0usize;
        let mut saw_positive_rps = false;
        while let Some(event) = progress_rx.recv().await {
            events += 1;
            if event.current_rps > 0.0 {
                saw_positive_rps = true;
            }
            if event.is_finished {
                assert_eq!(event.total_requests, event.successful_requests);
                assert!(event.percentiles.p50_ms <= event.percentiles.p95_ms);
                assert!(event.percentiles.p95_ms <= event.percentiles.p99_ms);
                assert!(event.bytes_per_second > 0.0);
            }
        }

        let final_event = runner.await.expect("engine task joined").expect("engine succeeded");
        assert!(final_event.is_finished);
        assert_eq!(final_event.test_id, "test-stream");
        assert!(final_event.total_requests > 0);
        assert!(events > 0, "progress events must be streamed");
        assert!(saw_positive_rps, "at least one window must report RPS > 0");
        drop(cancel_tx);
    }

    #[tokio::test]
    async fn cancel_halts_execution_immediately() {
        let service = LoadTestService::new(
            Arc::new(FixedHttpClient {
                status: 200,
                body_size: 16,
            }),
            Arc::new(NoopResolver),
        );
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (progress_tx, mut progress_rx) = mpsc::channel::<LoadTestProgressEventDto>(256);
        let latest_slot: LatestProgressSlot = Arc::new(Mutex::new(None));
        let (environment, globals) = empty_context();
        let run = LoadTestRun {
            config: sample_config(30),
            environment,
            globals,
            test_id: "test-cancel".to_string(),
            cancel_rx,
            progress_tx,
            latest_slot,
        };

        let runner = tokio::spawn(async move { service.run(run).await });

        // Let the engine spin up, then abort and measure halt latency.
        tokio::time::sleep(Duration::from_millis(300)).await;
        let halt_started = Instant::now();
        let _ = cancel_tx.send(true);
        let final_event = runner.await.expect("engine task joined").expect("engine succeeded");

        let halt_elapsed = halt_started.elapsed();
        assert!(
            halt_elapsed < Duration::from_millis(100),
            "cancellation must halt workers in < 100ms, took {halt_elapsed:?}"
        );
        assert!(final_event.is_finished);
        // Engine ran ~300ms pre-cancel; the exact count depends on machine
        // speed, so only assert the run produced work and then stopped.
        assert!(final_event.total_requests > 0);
        // Drain remaining events so the channel does not leak warnings.
        while progress_rx.try_recv().is_ok() {}
    }

    #[tokio::test]
    async fn transport_errors_count_as_failures() {
        struct FailingClient;

        #[async_trait::async_trait]
        impl HttpClientPort for FailingClient {
            async fn send(&self, _request: HttpRequest) -> Result<crate::domain::models::HttpResponse, DomainError> {
                Err(DomainError::NetworkError("connection refused".to_string()))
            }
        }

        let service = LoadTestService::new(Arc::new(FailingClient), Arc::new(NoopResolver));
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (progress_tx, _progress_rx) = mpsc::channel::<LoadTestProgressEventDto>(256);
        let latest_slot: LatestProgressSlot = Arc::new(Mutex::new(None));
        let (environment, globals) = empty_context();
        let run = LoadTestRun {
            config: sample_config(1),
            environment,
            globals,
            test_id: "test-failures".to_string(),
            cancel_rx,
            progress_tx,
            latest_slot,
        };

        let runner = tokio::spawn(async move { service.run(run).await });

        let final_event = runner.await.expect("engine joined").expect("engine succeeded");
        assert!(final_event.total_requests > 0);
        assert_eq!(final_event.total_requests, final_event.failed_requests);
        assert!(final_event
            .status_codes
            .iter()
            .any(|status| status.code == 0 && status.count == final_event.failed_requests));
        drop(cancel_tx);
    }
}
