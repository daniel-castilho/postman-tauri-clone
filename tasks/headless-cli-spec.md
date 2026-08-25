# Headless Automation & CLI — Spec (Phase 18)

**Companions:** `headless-cli-backlog.md`
**Epic Goal:** Run collections and test suites straight from the terminal so Tyny Pulse fits automated pipelines (CI/CD) without opening the desktop UI.

**Version basis:** `tyny-pulse 0.1.0`

---

## 1. Problem Statement

All request execution currently lives behind the Tauri IPC boundary: the binary always boots the desktop shell (`tauri::Builder`) and every workflow requires manual interaction. There is no way to:

1. Execute a collection unattended (nightly smoke tests, pre-deploy checks).
2. Emit machine-readable reports consumable by CI systems.
3. Fail a pipeline step when API assertions break.

---

## 2. Solution Overview

Add a **headless execution mode** to the existing binary. When the first CLI argument is a recognized subcommand, `main()` branches *before* the Tauri builder starts: the process never creates a window, wires the same infrastructure adapters manually, drives the existing `RunCollectionUseCase`, prints a human summary to stdout, optionally writes machine-readable reports, and exits with a pipeline-friendly status code.

Clean-Architecture payoff: application services and ports have zero Tauri coupling, so the CLI is just one more thin presentation adapter next to the Tauri command handlers.

```text
terminal ──> presentation::cli (parse args, orchestrate)
                 | reuses
                 v
     application::commands::run_collection::RunCollectionUseCase
                 | drives
                 v
 ReqwestHttpClientAdapter . RealVariableResolver . QuickJsScriptRunner
```

---

## 3. Command-Line Contract

```text
tyny-pulse run <collection.json> [options]

Options:
  --env <path>          Environment JSON file to load (optional)
  --globals <path>      Global variables JSON file to load (optional)
  --var <key=value>     Override/inject an environment variable (repeatable)
  --report <path>       Write a report file (extension decides writer)
  --format <json|junit> Explicit report format when --report has no known
                        extension (default: inferred, fallback json)
  -h, --help            Print usage and exit 0
  -V, --version         Print version and exit 0
```

### Input files

| File | Shape | Producer |
| :--- | :--- | :--- |
| Collection JSON | `domain::models::Collection` | App export / `export_workspace` |
| Environment JSON | `domain::models::Environment` | App environments storage |
| Globals JSON | `domain::models::GlobalVariables` | App globals storage |

Inputs deserialize with plain `serde_json` — no new file formats are introduced. `--var` overrides are applied onto the environment runtime map after loading (creating entries when unknown), matching `Environment::apply_runtime_map`.

### Exit codes

| Code | Meaning |
| :--- | :--- |
| `0` | Run finished; all tests passed |
| `1` | Run finished; at least one test failed |
| `2` | Usage or input error (bad flags, unreadable/malformed JSON) |
| `3` | Domain/runtime error surfaced by a use case |

---

## 4. Reports

### 4.1 JSON

Envelope wrapping the existing `CollectionRunReport` contract plus run metadata:

```json
{
  "tool": "tyny-pulse",
  "version": "0.1.0",
  "generatedAt": "<RFC3339>",
  "collection": "<name>",
  "summary": { "totalRequests": 12, "totalTests": 30, "passedTests": 29, "failedTests": 1 },
  "results": [ /* RequestRunResult[] unchanged */ ]
}
```

### 4.2 JUnit XML

Hand-rolled writer (no new dependencies). Mapping:

| JUnit element | Source |
| :--- | :--- |
| `<testsuites>` | Whole run |
| `<testsuite name="request" tests failures>` | Each `RequestRunResult` |
| `<testcase name>` | Each `TestResult` |
| `<failure message>` | `TestResult.error` when `passed == false` |

All text nodes/values are XML-escaped (`& < > " '`).

Report writers live in `infrastructure/reporting/` as pure functions over domain types, unit-tested without IO.

---

## 5. Architecture Placement

| Concern | Module |
| :--- | :--- |
| Arg parsing + orchestration + stdout summary + exit policy | `src-tauri/src/presentation/cli.rs` (pure logic unit-tested; no Tauri imports) |
| Report writers (JSON envelope, JUnit XML) | `src-tauri/src/infrastructure/reporting/` |
| Branch point | `main()`: subcommand present → tokio runtime + headless path → `std::process::exit(code)`; otherwise GUI as today |

Dependency rule holds: `presentation/cli.rs` touches only `application` + `domain`; `infrastructure/reporting` touches only `domain`.

---

## 6. Non-Goals (this phase)

- No watch/re-run loop, no parallel collection shards.
- No new binary target yet (single binary keeps distribution simple).
- gRPC requests still hit the mock client adapter (existing tech debt).
- No secret-vault decryption in CLI mode (secrets stay app-scope for now).

---

## 7. Known Limitations / Deferred Debt

1. **Windows GUI subsystem:** release builds set `windows_subsystem = "windows"`, so stdout may not attach when invoked from some Windows shells. Mitigation deferred: a future dedicated `tyny-pulse-cli` bin target without that attribute (tracked in AGENTS.md debt matrix).
2. **No `clap`:** arg parsing is hand-rolled std-only to respect the no-new-dependencies rule; revisit if the surface grows beyond Phase 18 scope.
