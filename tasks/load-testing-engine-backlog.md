# Tokio Load Testing Engine & Real-Time UI Charts — Backlog (P4)

**Companions:** `load-testing-engine-spec.md` · `load-testing-engine-implementation-sequence.md` · `ai-software-engineer-prompt-load-testing-engine.md`  
**Epic Goal:** Build a high-performance Tokio multi-threaded load testing engine in Rust with `mpsc` single-writer metric aggregation and real-time streaming charts in React 19.

**MVP Scope:** Stories S1–S9

---

## Story Map

```text
CORE ENGINE & WORKER POOL
S1 Implement LoadTestService core struct & configuration validation
S2 Build Tokio worker task loop & reqwest HTTP execution engine
S3 Implement lock-free mpsc metric aggregator & percentile calculator

IPC COMMANDS & EVENT STREAMING
S4 Implement Tauri IPC commands (start_load_test, stop_load_test)
S5 Stream load_test_progress events at 200ms intervals over Tauri IPC
S6 Export TypeScript DTO bindings for LoadTest progress and config via ts-rs

REACT 19 VISUAL CHARTS & PANEL
S7 Build Load Testing Panel layout (VUs slider, duration, request picker)
S8 Implement real-time animated charts (RPS over time, Latency percentiles, Status codes)
S9 Add exportable JSON and Markdown load test summary report generator
```

---

## Stories Breakdown

| ID | Story Title | Priority | Target Modules / Components | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **S1** | Implement `LoadTestService` core struct & configuration validation | Must | `src-tauri/src/application/services/load_test_service.rs` | Handles VUs (1–500), duration, ramp-up, and target request setup |
| **S2** | Build Tokio worker task loop & reqwest HTTP execution engine | Must | `src-tauri/src/application/services/load_test_service.rs` | Spawns Tokio tasks executing concurrent async HTTP requests |
| **S3** | Implement lock-free `mpsc` metric aggregator & percentile calculator | Must | `src-tauri/src/application/services/load_test_service.rs` | Single-writer task calculating RPS, p50, p90, p95, p99 percentiles |
| **S4** | Implement Tauri IPC commands (`start_load_test`, `stop_load_test`) | Must | `src-tauri/src/application/commands/load_test.rs`, `main.rs` | Start and graceful cancellation via `CancellationToken` |
| **S5** | Stream `load_test_progress` events at 200ms intervals over Tauri IPC | Must | `src-tauri/src/application/commands/load_test.rs` | Emits sampled progress events without flooding the IPC bridge |
| **S6** | Export TypeScript DTO bindings for LoadTest progress and config | Must | `src-tauri/src/application/commands/export_ts_bindings.rs` | Generates TS interfaces in `src/types/generated/` |
| **S7** | Build Load Testing Panel layout (VUs slider, duration, request picker) | Must | `src/components/LoadTestingPanel/LoadTestingPanel.tsx` | UI configuration panel with live execution controls |
| **S8** | Implement real-time animated charts (RPS, Latency p50/p95/p99, Status codes) | Must | `src/components/LoadTestingPanel/LoadTestCharts.tsx` | Live chart rendering using Framer Motion / Recharts / Canvas |
| **S9** | Add exportable JSON & Markdown load test summary report generator | Must | `src/components/LoadTestingPanel/LoadTestReport.tsx` | Export formatted summary reports for test runs |

---

## Definition of Done (Epic)

- [ ] S1–S9 completed and verified.
- [ ] Tokio load testing engine executes concurrent HTTP stress runs in Rust without UI freezes.
- [ ] Real-time `load_test_progress` events stream sampled stats every 200ms.
- [ ] Real-time RPS, Latency (p50, p95, p99), and Status Code charts render fluidly in React 19.
- [ ] Emergency stop (`stop_load_test`) halts all active worker tasks in < 100ms.
- [ ] `cargo test` and `npm run build` pass 100% green.
