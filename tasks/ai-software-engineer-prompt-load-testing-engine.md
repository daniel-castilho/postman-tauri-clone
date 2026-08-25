# AI Software Engineer Prompt — Tokio Load Testing Engine & Real-Time UI Charts (P4)

**Status:** Draft for implementation — Concurrent Stress Testing & Analytics Epic.
**Target:** Build a high-performance Tokio multi-threaded load testing engine in Rust with `mpsc` single-writer metric aggregation and real-time streaming charts in React 19.
**Package / Scope:** `src-tauri/src/application/services/load_test_service.rs`, `src-tauri/src/application/commands/load_test.rs`, `src/components/LoadTestingPanel/`

You implement the multi-threaded Tokio load testing engine and real-time analytics charts for **Tyny Pulse** so that developers can execute high-concurrency API stress tests, measure RPS, throughput, and latency percentiles (p50, p90, p95, p99), and monitor live visual charts during execution.

---

## Sources of Truth (Read in Order)

1. `AGENTS.md`
2. `docs/coding-standards.md` · `docs/testing-playbook.md` · `docs/data-model-decisions.md`
3. `tasks/load-testing-engine-spec.md` — Technical Specification
4. `tasks/load-testing-engine-backlog.md` — User Stories Map (S1–S9)
5. `tasks/load-testing-engine-implementation-sequence.md` — Step-by-Step Execution Sequence
6. Reference: `src-tauri/src/application/ports/http_client.rs`, `src-tauri/src/domain/models.rs`, `src/store/workspaceStore.ts`

---

## Goal

Bring **Tyny Pulse** to 100% load testing readiness with native Rust Tokio multi-threading and live frontend visualization:

- Build `LoadTestService` in `src-tauri/src/application/services/load_test_service.rs` spawning $N$ concurrent Tokio worker tasks driving HTTP request iterations via `reqwest`.
- Implement lock-free, zero-contention metric aggregation using a Tokio `mpsc` (Multi-Producer, Single-Consumer) channel to collect per-request latency samples without Mutex contention across threads.
- Emit real-time progress events over Tauri IPC (`load_test_progress`) at regular sampling intervals (default: every 200ms) containing RPS, active Virtual Users (VUs), throughput (bytes/sec), error counts, and calculated latency percentiles (p50, p90, p95, p99).
- Expose Tauri IPC commands (`start_load_test`, `stop_load_test`, `get_load_test_status`) returning strongly-typed DTOs annotated with `#[derive(TS)]`.
- Build an interactive React 19 Load Testing Panel (`src/components/LoadTestingPanel/`) with:
  - Configuration controls: VUs slider (1–500), duration (seconds), ramp-up time, request selector.
  - Real-time animated charts: RPS over time, Latency percentiles (p50, p95, p99), HTTP Status Code breakdown.
  - Start, Stop, and Emergency Abort controls.
  - Exportable execution summary reports (JSON and Markdown).

---

## Non-Negotiable Rules

- **Lock-Free Aggregation:** Tokio worker tasks must send latency samples to a dedicated aggregator task via an `mpsc` channel. Do not wrap shared latency vectors in `Arc<Mutex<Vec<u128>>>` across high-concurrency worker threads.
- **Non-Blocking UI Thread:** All load test worker loops and metric aggregations must execute asynchronously in Tokio task pools. Tauri IPC events must stream sampled stats without flooding the frontend event queue.
- **Graceful Cancellation:** Calling `stop_load_test` must send an immediate cancellation signal (`tokio::sync::watch` or `CancellationToken`) to all active worker tasks, stopping execution within < 100ms.
- **Accurate Percentiles:** Latency percentiles (p50, p90, p95, p99) must be computed using accurate histogram algorithms or sorted sample windows (`hdrhistogram` or fast quantile algorithms).
- **English Only:** All identifiers, attributes, IPC commands, UI labels, log messages, test functions, and commit messages must be in English.
- **Zero Behavior Change:** Core HTTP single request execution, QuickJS scripting, CLI runner, and Git panel functionality must remain 100% untouched.

---

## Definition of Done (Epic)

- [ ] `LoadTestService` implemented in Rust spawning Tokio worker tasks over `mpsc` aggregation channels.
- [ ] Real-time `load_test_progress` Tauri IPC events emitted at regular 200ms intervals.
- [ ] Tauri IPC commands registered and `ts-rs` bindings exported to `src/types/generated/`.
- [ ] React 19 Load Testing Panel with real-time RPS and latency percentile charts implemented.
- [ ] Graceful abort mechanism (`stop_load_test`) verified.
- [ ] Rust unit & integration tests (`cargo test`) and frontend build (`npm run build`) pass 100% green.

Start at **Step 0** of `tasks/load-testing-engine-implementation-sequence.md`. If any instruction or load testing scope is unclear, **stop and ask**.
