# Headless CLI Runner (`tyny-cli`) — Implementation Sequence (P2)

**Companions:** `headless-cli-spec.md` · `headless-cli-backlog.md` · `ai-software-engineer-prompt-headless-cli.md`  
**Rule:** Finish each step's "Done when" before moving to the next. Do not invent scope.

---

## Step 0 — Analysis & Interface Review

1. Review entry point `src-tauri/src/main.rs` and identify where `std::env::args()` can be intercepted before `tauri::Builder::default()` starts.
2. Review `RunCollectionUseCase` interface and dependencies (`HttpClientPort`, `ScriptEnginePort`, `FileSystemPort`).
3. Ensure application layer services do not depend on Tauri Webview or IPC channels.

**Done when:** Interception point in `main.rs` is identified and application layer headless independence is verified.

---

## Step 1 — CLI Argument Parser (`presentation/cli.rs`)

1. Create module `src-tauri/src/presentation/cli.rs`.
2. Implement CLI argument parser handling:
   - Subcommands: `run <collection_path.json>`
   - Flags: `--env <path>`, `--globals <path>`, `--var <key=value>` (repeatable), `--report <path>`, `--format <json|junit>`, `--help`, `--version`.
3. Add unit tests in `cli.rs` verifying flag parsing and help display.

**Done when:** `cargo test` passes CLI argument parsing unit tests cleanly.

---

## Step 2 — Main Entry Interceptor Wiring

1. Modify `src-tauri/src/main.rs` to parse CLI arguments prior to Tauri application initialization:
   ```rust
   let args: Vec<String> = std::env::args().collect();
   if presentation::cli::is_cli_mode(&args) {
       let exit_code = presentation::cli::run_headless(args);
       std::process::exit(exit_code);
   }
   ```
2. Test running `cargo run --manifest-path src-tauri/Cargo.toml -- --help` from terminal to verify CLI mode interceptor works without spawning GUI window.

**Done when:** Running CLI flags in terminal bypasses Tauri GUI startup.

---

## Step 3 — Variable Resolution & Overrides

1. Read environment JSON file if `--env` is supplied.
2. Read global variables JSON file if `--globals` is supplied.
3. Parse repeatable `--var key=value` flags and override environment variable map.
4. Pass resolved variables into `RunCollectionUseCase`.

**Done when:** CLI variable overrides correctly update the execution context.

---

## Step 4 — Headless Reporting Engine (`infrastructure/reporting/`)

1. Create module `src-tauri/src/infrastructure/reporting/mod.rs`.
2. Implement `json_reporter.rs`: serializes `CollectionRunReport` into formatted JSON output file.
3. Implement `junit_reporter.rs`: serializes `CollectionRunReport` into standard JUnit XML output file with `<testsuites>`, `<testsuite>`, `<testcase>`, and `<failure>` nodes.
4. Add unit tests verifying JSON envelope and XML schema generation.

**Done when:** Both JSON and JUnit XML reporting modules pass unit tests cleanly.

---

## Step 5 — Exit Code Mapping & Error Handling

1. Implement exit code evaluation logic in `presentation/cli.rs`:
   - `0`: All collection requests and test assertions passed.
   - `1`: One or more assertions failed (`failedAssertions > 0`).
   - `2`: File not found, invalid argument syntax, or malformed JSON.
   - `3`: Network connection failure, QuickJS panic, or domain error.
2. Ensure exit codes are returned directly via `std::process::exit(code)`.

**Done when:** Process exit codes correctly reflect execution outcomes across all scenarios.

---

## Step 6 — Integration Verification & CI Template

1. Create GitHub Actions template `.github/workflows/tyny-cli-ci.yml`.
2. Run end-to-end smoke path against local mock HTTP server:
   ```bash
   cargo run --manifest-path src-tauri/Cargo.toml -- run ./tests/fixtures/sample_collection.json --report ./target/report.xml --format junit
   ```
3. Verify that `./target/report.xml` is created and contains valid JUnit XML syntax.
4. Update `docs/progress.md`, `README.md`, and `docs/testing-playbook.md` with CLI runner details.

**Done when:** Epic Definition of Done is fully met.

---

## Smoke Path

1. Terminal execution `tyny-cli run <collection.json>` runs headlessly without GUI.
2. Passed collection run returns exit code `0`.
3. Collection run with failing assertion returns exit code `1`.
4. Invalid file path returns exit code `2`.
5. Report file `--report results.xml` contains valid JUnit XML syntax.
