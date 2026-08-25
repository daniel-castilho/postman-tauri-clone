# Technical Specification — Headless CLI Runner (`tyny-cli`) (P2)

**Status:** Draft for Implementation  
**Epic Focus:** Native Rust command-line collection execution engine with JSON & JUnit XML reporting  
**Companions:** `headless-cli-backlog.md` · `headless-cli-implementation-sequence.md` · `ai-software-engineer-prompt-headless-cli.md`

**Official Domain:** [https://tyny.ca](https://tyny.ca) | **App ID:** `ca.tyny.pulse`

---

## 1. Purpose & Scope

Provide a lightweight, high-performance, headless CLI execution engine for **Tyny Pulse** that allows developers to run collection test suites, evaluate JavaScript assertions, and output standardized CI/CD test reports in terminal environments without needing a GUI desktop session.

### In Scope (P2)
- Command-line argument parser & dispatcher in `src-tauri/src/presentation/cli.rs`.
- Subcommand interface: `tyny-cli run <collection_path.json>` with optional flags.
- Support for CLI flags:
  - `--env <path>` / `-e`: Path to environment variables JSON file.
  - `--globals <path>` / `-g`: Path to global variables JSON file.
  - `--var <key=value>` / `-v`: Overriding variable key-value pairs (repeatable).
  - `--report <path>` / `-r`: Output file path for test report.
  - `--format <json|junit>` / `-f`: Report format (defaults based on file extension `.xml` / `.json`).
- Headless execution over existing application services (`RunCollectionUseCase`).
- Report generation:
  - **JSON Report:** Complete execution metadata envelope including requests, responses, script logs, and assertions.
  - **JUnit XML Report:** XML schema compatible with CI/CD test visualizers (GitHub Actions, GitLab CI, Jenkins).
- Standardized process exit codes (`0`, `1`, `2`, `3`).
- Documentation & CI pipeline templates.

### Out of Scope
- Full interactive TUI (Terminal User Interface) with ncurses/ratatui.
- Real-time cloud telemetry streaming.
- Executing GUI desktop commands in CLI mode.

---

## 2. Architecture & Headless Execution Flow

```
                                  COMMAND LINE ARGS
                                          │
                                          ▼
                         ┌─────────────────────────────────┐
                         │   presentation/cli.rs Dispatch  │
                         └────────────────┬────────────────┘
                                          │
                        ┌─────────────────┴─────────────────┐
                        │ Is CLI subcommand (e.g. `run`)?  │
                        └────────┬───────────────────┬──────┘
                                 │ YES               │ NO
                                 ▼                   ▼
                  ┌─────────────────────────────┐ ┌──────────────────────────┐
                  │ Headless CLI Execution Mode │ │  Launch Tauri Desktop    │
                  └──────────────┬──────────────┘ │  App GUI (main.rs)       │
                                 │                └──────────────────────────┘
                                 ▼
                  ┌─────────────────────────────┐
                  │     Application Layer       │
                  │  (RunCollectionUseCase)     │
                  └──────────────┬──────────────┘
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
   ┌──────────────────────────┐    ┌──────────────────────────┐
   │   Reqwest HTTP Adapter   │    │  QuickJS Sandbox Engine  │
   └─────────────┬────────────┘    └─────────────┬────────────┘
                 │                               │
                 └───────────────┬───────────────┘
                                 │
                                 ▼
                  ┌─────────────────────────────┐
                  │    infrastructure/reporting │
                  │     (JSON / JUnit XML)      │
                  └──────────────┬──────────────┘
                                 │
                                 ▼
                  ┌─────────────────────────────┐
                  │    Process Exit Code        │
                  │  (0 = Pass, 1 = Fail, ...)  │
                  └─────────────────────────────┘
```

---

## 3. CLI Command & Flag Specification

### Command Syntax
```bash
tyny-cli run <COLLECTION_PATH> [OPTIONS]
```

### Flags & Options
| Flag (Short / Long) | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `-e`, `--env <PATH>` | String | Path to workspace environment JSON file | Optional |
| `-g`, `--globals <PATH>` | String | Path to global variables JSON file | Optional |
| `-v`, `--var <KEY=VALUE>` | String (Repeatable) | Variable overrides (e.g. `-v baseUrl=https://api.tyny.ca`) | Optional |
| `-r`, `--report <PATH>` | String | Output report destination file path | Optional |
| `-f`, `--format <FORMAT>` | Enum (`json`, `junit`) | Report output format | Inferred from file extension |
| `-h`, `--help` | Flag | Output CLI help information | N/A |
| `-V`, `--version` | Flag | Output version information | N/A |

### Example CLI Usage
```bash
# Basic execution
tyny-cli run ./collections/auth_suite.json

# Execution with environment, CLI variable override, and JUnit report
tyny-cli run ./collections/auth_suite.json \
  --env ./environments/staging.json \
  --var baseUrl=https://staging.tyny.ca \
  --report ./reports/results.xml \
  --format junit
```

---

## 4. Exit Code Contract

| Code | Meaning | Condition |
| :-: | :--- | :--- |
| **`0`** | **SUCCESS** | All requests completed and 100% of test assertions passed. |
| **`1`** | **TEST_FAILURE** | Collection executed successfully, but 1 or more test assertions failed. |
| **`2`** | **USAGE_ERROR** | Invalid CLI arguments, non-existent input files, or invalid JSON syntax. |
| **`3`** | **EXECUTION_ERROR** | Unhandled domain error, network connectivity failure, or QuickJS panic. |

---

## 5. Report Specifications

### 5.1 JSON Report Structure
Full execution snapshot serialized as JSON:
```json
{
  "version": "1.0",
  "summary": {
    "totalRequests": 10,
    "passedRequests": 9,
    "failedRequests": 1,
    "totalAssertions": 25,
    "passedAssertions": 24,
    "failedAssertions": 1,
    "durationMs": 1420
  },
  "results": [...]
}
```

### 5.2 JUnit XML Report Structure
XML compliant with JUnit schema for CI test visualizers:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Tyny Pulse Collection Run" tests="25" failures="1" errors="0" time="1.420">
  <testsuite name="Authentication Flow" tests="3" failures="0" errors="0" time="0.210">
    <testcase name="Status code is 200 OK" classname="Auth.Login" time="0.085"/>
    <testcase name="Returns valid JWT token" classname="Auth.Login" time="0.085"/>
  </testsuite>
</testsuites>
```

---

## 6. Testing & Validation Requirements

1. **CLI Argument Unit Tests:** Test flag parsing (`--env`, `--var`, `--report`, `--format`) in `presentation/cli.rs`.
2. **Headless Execution Test:** Execute collections headlessly against a mock HTTP server without initializing Tauri Webview windows.
3. **Reporter Validation Tests:** Verify JSON envelope validity and JUnit XML syntax correctness.
4. **Exit Code Tests:** Assert process exit codes `0`, `1`, `2`, `3` across appropriate test scenarios.

---

## 7. Definition of Done

- [x] CLI subcommand dispatcher implemented in `src-tauri/src/presentation/cli.rs`.
- [x] Application layer `RunCollectionUseCase` executed headlessly without Tauri Webview.
- [x] JSON and JUnit XML reporters implemented in `src-tauri/src/infrastructure/reporting/`.
- [x] Process exit codes (`0`, `1`, `2`, `3`) verified.
- [x] GitHub Actions CI integration template shipped in `.github/workflows/tyny-cli-ci.yml`.
- [x] Both `cargo test` and `npm run build` pass 100% green.

## 8. Implementation Notes (as-built)

- The dedicated binary lives at `src-tauri/src/bin/tyny-cli.rs` and consumes the library crate (`tyny_pulse_lib`), guaranteeing execution parity with the desktop app while avoiding the GUI subsystem attribute on Windows release builds.
- The desktop `tyny-pulse` binary keeps full CLI parity in debug builds via the same `is_cli_mode()` / `run_headless()` entry points.
- JSON reports include additive metadata fields (`tool`, `generatedAt`, `collection`) alongside the spec §5.1 contract; consumers relying on documented keys are unaffected.
- A request counts as "passed" in `passedRequests` when none of its assertions failed (requests without scripts count as passed).
