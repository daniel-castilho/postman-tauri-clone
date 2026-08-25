# Headless CLI Runner (`tyny-cli`) — Backlog (P2)

**Companions:** `headless-cli-spec.md` · `headless-cli-implementation-sequence.md` · `ai-software-engineer-prompt-headless-cli.md`  
**Epic Goal:** Build a standalone, headless CLI execution engine in Rust that reuses the application layer to execute collections in terminal environments and CI/CD pipelines with JSON and JUnit XML reporting.

**MVP Scope:** Stories S1–S8

---

## Story Map

```text
ARGUMENT PARSING & DISPATCH
S1 Implement CLI argument parser in presentation/cli.rs
S2 Wire CLI subcommand interceptor before Tauri GUI initialization

HEADLESS CORE EXECUTION
S3 Adapt RunCollectionUseCase for headless execution over Reqwest and QuickJS
S4 Implement CLI variable overrides (--var key=value, --env, --globals)

REPORTING & OUTPUT
S5 Implement JSON execution report writer in infrastructure/reporting/
S6 Implement JUnit XML report writer in infrastructure/reporting/

EXIT CODES & CI INTEGRATION
S7 Implement standardized process exit codes (0, 1, 2, 3)
S8 Create GitHub Actions CI template (.github/workflows/tyny-cli-ci.yml)
```

---

## Stories Breakdown

| ID | Story Title | Priority | Target Modules / Components | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **S1** ✅ | Implement CLI argument parser in `presentation/cli.rs` | Must | `src-tauri/src/presentation/cli.rs` | Parse `run`, `--env/-e`, `--globals/-g`, `--var/-v`, `--report/-r`, `--format/-f`, `--help` |
| **S2** ✅ | Wire CLI subcommand interceptor before Tauri GUI initialization | Must | `src-tauri/src/main.rs` | `is_cli_mode()` + `run_headless()` per implementation sequence Step 2 |
| **S3** ✅ | Adapt `RunCollectionUseCase` for headless execution over Reqwest & QuickJS | Must | `src-tauri/src/application/services/` | Reused via library crate; zero Webview/Tauri IPC coupling |
| **S4** ✅ | Implement CLI variable overrides (`--var key=value`, `--env`, `--globals`) | Must | `src-tauri/src/presentation/cli.rs` | Merge environment files with command-line variable overrides |
| **S5** ✅ | Implement JSON execution report writer in `infrastructure/reporting/` | Must | `src-tauri/src/infrastructure/reporting/json_reporter.rs` | Spec §5.1 envelope: version, request/assertion summary, durationMs |
| **S6** ✅ | Implement JUnit XML report writer in `infrastructure/reporting/` | Must | `src-tauri/src/infrastructure/reporting/junit_reporter.rs` | testsuites/testsuite/testcase/failure with errors + classname + time |
| **S7** ✅ | Implement standardized process exit codes (`0`, `1`, `2`, `3`) | Must | `src-tauri/src/presentation/cli.rs` | Covered by integration tests incl. real HTTP mock server |
| **S8** ✅ | Create GitHub Actions CI template (`.github/workflows/tyny-cli-ci.yml`) | Must | `.github/workflows/tyny-cli-ci.yml`, `README.md` | workflow_dispatch example building tyny-cli and running the fixture |

---

## Definition of Done (Epic)

- [x] S1–S8 completed and verified.
- [x] Collection execution via CLI completes headlessly in terminal without GUI window.
- [x] JSON and JUnit XML reports generated accurately.
- [x] Exit codes (`0`, `1`, `2`, `3`) match the specification contract.
- [x] Rust unit and integration tests (`cargo test`) pass 100% green (66 unit + 6 integration).
- [x] Both `cargo test` and `npm run build` pass without warnings or errors.
