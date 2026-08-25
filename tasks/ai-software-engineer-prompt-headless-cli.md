# AI Software Engineer Prompt — Headless CLI Runner (`tyny-cli`) (P2)

**Status:** Draft for implementation — Headless CLI & Automated Collection Execution Epic.
**Target:** Build a standalone, headless CLI execution engine (`tyny-cli` / `presentation/cli.rs`) in Rust that reuses the application layer to execute collections in terminal environments and CI/CD pipelines without launching the Tauri GUI.
**Package / Scope:** `src-tauri/src/presentation/cli.rs`, `src-tauri/src/infrastructure/reporting/`

You implement the headless CLI runner and automated reporting engine for **Tyny Pulse** so that developers can run API test collections in terminal environments, generate standard JSON and JUnit XML reports, and integrate test suites directly into CI/CD pipelines.

---

## Sources of Truth (Read in Order)

1. `AGENTS.md`
2. `docs/coding-standards.md` · `docs/testing-playbook.md` · `docs/data-model-decisions.md`
3. `tasks/headless-cli-spec.md` — Technical Specification
4. `tasks/headless-cli-backlog.md` — User Stories Map (S1–S8)
5. `tasks/headless-cli-implementation-sequence.md` — Step-by-Step Execution Sequence
6. Reference: `src-tauri/src/application/services/`, `src-tauri/src/presentation/`, `src-tauri/src/main.rs`

---

## Goal

Bring **Tyny Pulse** to 100% headless CI/CD readiness with a native Rust CLI execution engine:

- Create command-line dispatch in `presentation/cli.rs` that intercepts CLI subcommands (`tyny-cli run <collection.json>`, `--env`, `--globals`, `--var key=value`, `--report`, `--format json|junit`) before launching the Tauri GUI builder.
- Reuse the pure application layer (`RunCollectionUseCase`) wired over Reqwest HTTP, QuickJS sandbox, and FileSystem adapters with **zero Tauri Webview/GUI coupling**.
- Build lightweight, dependency-free reporting adapters for **JSON** metadata envelopes and **JUnit XML** test result files (`CollectionRunReport`).
- Standardize process exit codes:
  - `0`: All tests passed successfully.
  - `1`: One or more assertion tests failed.
  - `2`: Invalid CLI arguments or missing files.
  - `3`: Domain, network, or execution errors.
- Ship a GitHub Actions workflow example (`.github/workflows/tyny-cli-ci.yml`) demonstrating headless execution.

---

## Non-Negotiable Rules

- **Zero Tauri Coupling in Headless Mode:** Executing CLI subcommands must not initialize Webview windows or Tauri GUI event loops.
- **Application Layer Reuse:** The CLI runner must consume the exact same `RunCollectionUseCase` application service used by the desktop app to guarantee 100% execution parity.
- **No Unapproved Heavy Dependencies:** Parse CLI flags using std/lightweight argument parsing without adding bloated third-party CLI dependencies if possible.
- **Deterministic Reporting:** JUnit XML output must strictly validate against the standard JUnit XSD schema (testsuites, testsuite, testcase, failure tags).
- **English Only:** All identifiers, attributes, CLI messages, logs, test functions, and commit messages must be in English.
- **Zero Behavior Change:** Existing Tauri desktop app functionality must remain 100% untouched.

---

## Definition of Done (Epic)

- [ ] CLI argument dispatcher in `presentation/cli.rs` handling `run`, `--env`, `--globals`, `--var`, `--report`, `--format`.
- [ ] Headless execution of collections using `RunCollectionUseCase` over Reqwest/QuickJS adapters.
- [ ] JSON and JUnit XML reporters implemented in `infrastructure/reporting/`.
- [ ] Exit codes (0, 1, 2, 3) verified across passing, failing, and error paths.
- [ ] Unit & integration test suite added in Rust (`cargo test`).
- [ ] Both `cargo test` and `npm run build` pass without warnings or errors.

Start at **Step 0** of `tasks/headless-cli-implementation-sequence.md`. If any instruction or CLI flag scope is unclear, **stop and ask**.
