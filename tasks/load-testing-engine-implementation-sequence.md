# Tokio Load Testing Engine & Real-Time UI Charts — Implementation Sequence (P4)

**Companions:** `load-testing-engine-spec.md` · `load-testing-engine-backlog.md` · `ai-software-engineer-prompt-load-testing-engine.md`  
**Rule:** Finish each step's "Done when" before moving to the next. Do not invent scope.

---

## Step 0 — Analysis & Dependency Verification

1. Review HTTP client adapter (`HttpClientPort` / `reqwest`) and confirm thread safety (`Send + Sync`).
2. Verify Tokio features in `Cargo.toml` (`tokio = { version = "1", features = ["full"] }`).
3. Confirm `hdrhistogram` or fast percentile calculation module is available.

**Done when:** Tokio thread safety and dependency prerequisites are verified.

---

## Step 1 — Domain DTOs & Percentile Types

1. Create/update DTO structs in `src-tauri/src/domain/models.rs`:
   - `LoadTestConfigDto` (`target_request`, `virtual_users`, `duration_seconds`, `ramp_up_seconds`, `timeout_ms`)
   - `LatencyPercentilesDto` (`p50_ms`, `p90_ms`, `p95_ms`, `p99_ms`, `min_ms`, `max_ms`, `mean_ms`)
   - `LoadTestProgressEventDto` (`test_id`, `elapsed_seconds`, `active_vus`, `current_rps`, `total_requests`, `percentiles`, etc.)
   - `StatusCodeCountDto` (`code`, `count`)
2. Annotate all DTOs with `#[derive(TS)]` and export paths to `../../src/types/generated/`.

**Done when:** Load test DTOs compile cleanly in Rust.

---

## Step 2 — Tokio Worker Pool & Single Writer `mpsc` Aggregator

1. Implement `src-tauri/src/application/services/load_test_service.rs`:
   - Define `Sample` struct (`duration_ms: u32`, `status_code: u16`, `bytes: usize`).
   - Create Tokio `mpsc::channel::<Sample>(10000)`.
   - Implement `aggregator_task`: receives samples, calculates RPS, throughput, and percentiles (p50, p90, p95, p99).
   - Implement `worker_task`: loops sending async HTTP requests via `reqwest` and pushing samples into `mpsc` sender.
2. Implement cancellation via `CancellationToken`.
3. Add unit tests in `load_test_service.rs` against a local mock HTTP server.

**Done when:** `cargo test` passes multi-threaded load test engine unit tests cleanly.

---

## Step 3 — Tauri IPC Commands & Event Streaming

1. Implement IPC command module `src-tauri/src/application/commands/load_test.rs`:
   - `start_load_test`: validates config, spawns `LoadTestService` in background Tokio task.
   - `stop_load_test`: triggers `CancellationToken` to halt workers immediately.
2. Stream `load_test_progress` events over Tauri IPC at 200ms intervals using `app_handle.emit()`.
3. Register commands in `src-tauri/src/main.rs`.
4. Update `src-tauri/src/application/commands/export_ts_bindings.rs` and run `cargo test export_ts_bindings`.

**Done when:** `cargo test` exports TypeScript bindings and IPC progress event streaming is verified.

---

## Step 4 — React 19 Load Testing Panel UI

1. Create directory `src/components/LoadTestingPanel/`.
2. Build `LoadTestingPanel.tsx`:
   - Target request picker (select request from active workspace).
   - VUs configuration slider (1–500), duration input, ramp-up time input.
   - Start / Stop control buttons with live status badges.
3. Integrate `LoadTestingPanel` into main layout navigation.

**Done when:** Load testing configuration form renders in the UI and sends IPC commands.

---

## Step 5 — Real-Time Animated Charts Component

1. Build `LoadTestCharts.tsx`:
   - Line chart for RPS over time.
   - Multi-line chart for Latency percentiles (p50, p95, p99).
   - Bar / Donut chart for HTTP Status Code distribution (2xx, 3xx, 4xx, 5xx).
2. Listen to `load_test_progress` IPC events and update chart dataset state seamlessly.

**Done when:** Real-time charts animate fluidly during active load test runs without UI lag.

---

## Step 6 — Final Verification & Smoke Path

1. Run full build & test suite:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   npm run build
   ```
2. Run desktop app in dev mode (`npm run tauri dev`).
3. Execute end-to-end smoke path:
   - Select target HTTP request.
   - Configure 50 VUs, 10 seconds duration.
   - Click Start Load Test → verify real-time RPS and Latency (p50/p95/p99) charts animate.
   - Click Stop Load Test → verify workers halt immediately (< 100ms).
   - Export summary Markdown report → verify metrics match execution.
4. Update `docs/progress.md`, `README.md`, and `docs/testing-playbook.md`.

**Done when:** Epic Definition of Done is fully met.

---

## Smoke Path

1. Tokio worker task pool executes 50+ concurrent async HTTP requests cleanly.
2. `load_test_progress` IPC events stream at 200ms without clogging Webview event loop.
3. Latency percentiles (p50, p90, p95, p99) calculate accurately.
4. Real-time charts render smoothly in React 19.
5. Emergency stop halts workers in < 100ms.
