# Technical Specification — Tokio Load Testing Engine & Real-Time UI Charts (P4)

**Status:** Draft for Implementation  
**Epic Focus:** Multi-threaded Tokio load testing engine in Rust with `mpsc` metric aggregation and real-time streaming charts in React 19  
**Companions:** `load-testing-engine-backlog.md` · `load-testing-engine-implementation-sequence.md` · `ai-software-engineer-prompt-load-testing-engine.md`

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Purpose & Scope

Provide a native, high-performance **API load and stress testing engine** directly inside **Tyny Pulse**. Developers can configure virtual user concurrency, execute multi-threaded HTTP load runs via Rust Tokio, measure throughput and latency percentiles (p50, p90, p95, p99), and visualize live performance charts without switching to external tools like k6 or JMeter.

### In Scope (P4)
- High-concurrency Tokio load testing service in `src-tauri/src/application/services/load_test_service.rs`.
- Lock-free, zero-contention metric aggregation using Tokio `mpsc` channels.
- Real-time streaming progress events over Tauri IPC (`load_test_progress`).
- Configuration parameters:
  - `virtual_users` (VUs): 1 to 500 concurrent workers.
  - `duration_seconds`: Test run duration (1s to 3600s).
  - `ramp_up_seconds`: Time window to gradually scale up active VUs.
  - `timeout_ms`: Per-request timeout limit.
  - `target_request`: HTTP method, URL, headers, and body payload.
- Real-time metric computations:
  - Requests Per Second (RPS) & Total Requests.
  - Latency percentiles: p50 (median), p90, p95, p99, Min, Max, Mean.
  - Throughput (Bytes/sec received).
  - Status code breakdown (2xx, 3xx, 4xx, 5xx, timeouts/errors).
- Tauri IPC commands: `start_load_test`, `stop_load_test`, `get_load_test_status`.
- DTOs annotated with `#[derive(TS)]` exported to `src/types/generated/`.
- React 19 Load Testing Panel UI (`src/components/LoadTestingPanel/`):
  - Configuration form & target selector.
  - Real-time animated charts (RPS over time, Latency percentiles, Status distribution).
  - Live progress counter cards & status badges.
  - Exportable Markdown / JSON summary reports.

### Out of Scope
- Multi-node distributed agent clustering (single desktop machine execution in P4).
- Distributed cloud load generators.

---

## 2. Architecture & Data Flow

```
  ┌─────────────────────────────────────────────────────────────┐
  │                 REACT 19 FRONTEND UI                        │
  │     src/components/LoadTestingPanel/ (Config & Charts)      │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
                   Tauri IPC Command / Events
                   (load_test_progress at ~200ms)
                                 │
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │               TAURI IPC COMMAND HANDLERS                    │
  │      src-tauri/src/application/commands/load_test.rs       │
  └──────────────────────────────┬──────────────────────────────┘
                                 │
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │              TOKIO LOAD TEST SERVICE ENGINE                 │
  │     src-tauri/src/application/services/load_test_service.rs │
  └──────┬───────────────────────┬───────────────────────┬──────┘
         │                       │                       │
         ▼                       ▼                       ▼
  ┌──────────────┐        ┌──────────────┐        ┌──────────────┐
  │ Worker Task 1│        │ Worker Task 2│ ...    │ Worker Task N│ (Tokio Tasks)
  └──────┬───────┘        └──────┬───────┘        └──────┬───────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │ (mpsc Channel - Lock-Free)
                                 ▼
  ┌─────────────────────────────────────────────────────────────┐
  │            SINGLE WRITER METRIC AGGREGATOR TASK             │
  │       (Computes RPS, Percentiles p50/p95/p99, Throughput)   │
  └─────────────────────────────────────────────────────────────┘
```

---

## 3. Data Models & IPC DTOs (`src-tauri/src/domain/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LoadTestConfigDto {
    pub target_request: HttpRequestDto,
    pub virtual_users: u32,
    pub duration_seconds: u64,
    pub ramp_up_seconds: u64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LatencyPercentilesDto {
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LoadTestProgressEventDto {
    pub test_id: String,
    pub elapsed_seconds: f64,
    pub active_vus: u32,
    pub current_rps: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub bytes_per_second: f64,
    pub percentiles: LatencyPercentilesDto,
    pub status_codes: Vec<StatusCodeCountDto>,
    pub is_finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct StatusCodeCountDto {
    pub code: u16,
    pub count: u64,
}
```

---

## 4. Performance & Lock-Free Design Rules

1. **Single Writer Pattern via `mpsc`:**
   Worker tasks send a minimal sample struct (`Sample { duration_ms: u32, status_code: u16, bytes: usize }`) over a Tokio `mpsc::channel(10000)` to a single aggregator task. No Mutex lock is ever acquired during the HTTP request loop.
2. **Sampling & Event Throttling:**
   The aggregator task collects samples and computes windowed percentiles every 200ms, emitting a single `load_test_progress` event to Tauri. This avoids flooding the frontend IPC bridge with thousands of individual per-request events.
3. **Graceful Cancellation:**
   A `tokio::sync::watch` or `tokio_util::sync::CancellationToken` is shared across all worker tasks. When `stop_load_test` is invoked, the token triggers, causing all worker loops to break immediately.

---

## 5. Testing & Validation Requirements

1. **Service Unit Tests:** Test `LoadTestService` against a local mock HTTP server (`axum` / `wiremock`), verifying RPS calculations and latency percentile accuracy.
2. **Cancellation Integration Test:** Verify that invoking `stop_load_test` halts worker execution in < 100ms.
3. **IPC & TS Bindings Verification:** Verify `LoadTestProgressEventDto` and `LoadTestConfigDto` bindings generated in `src/types/generated/`.
4. **Frontend Render Test:** Verify `npm run build` compiles React 19 load testing panel components cleanly.

---

## 6. Definition of Done

- [ ] `LoadTestService` implemented in Rust with Tokio multi-threading and `mpsc` lock-free channel aggregation.
- [ ] Real-time `load_test_progress` events emitted to Tauri Webview at 200ms intervals.
- [ ] IPC commands (`start_load_test`, `stop_load_test`) registered and `ts-rs` bindings exported.
- [ ] React 19 Load Testing Panel with real-time RPS, Latency (p50, p95, p99), and Status breakdown charts completed.
- [ ] `cargo test` and `npm run build` pass 100% green.
