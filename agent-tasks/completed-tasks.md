# completed-tasks.md

Log of all completed agentic coding work, appended in completion order
as tasks are dispatched from `agent-tasks.md`. Persists across sprints.

---

## Sprint s0 — Project Bootstrap

### Build Phase

- **A.1** (2026-05-19) — Edited root `Cargo.toml`: `[workspace]
  members = [".", "gui"]`. CLI-only install path preserved (`cargo
  install --path .` still installs only the root package).
- **B.1** (2026-05-19) — Created `gui/Cargo.toml` (package
  `oovra-gui`, edition 2024, rust-version 1.92, deps `egui = "0.34"`,
  `eframe = "0.34"` + persistence, `oovra = { path = ".." }`, and
  per-target wasm32 / non-wasm32 deps).
- **B.4** (2026-05-19) — Created `gui/src/main.rs`: native main
  (`eframe::run_native`, 720x480 window) and WASM main
  (`wasm_bindgen_futures::spawn_local` driving `eframe::WebRunner` on
  `#the_canvas_id`), cfg-gated.
- **B.2** (2026-05-19) — Created `gui/src/app.rs`: `OovraApp` (serde
  Deserialize/Serialize, eframe persistence wired) with a live
  `is_kebab_case` / `slugify` probe driven by user input — proves
  the gui crate calls real `oovra::header` functions at runtime. Also
  added `oovra::VERSION` constant in `src/lib.rs` so the gui can show
  the linked lib version.
- **B.3** (2026-05-19) — Created `gui/src/lib.rs`: `mod app; pub use
  app::OovraApp;` plus the standard lint warns.
- **B.5** (2026-05-19) — Created `gui/index.html`: Trunk web entry
  with `<link data-trunk rel="rust" data-bin="oovra-gui"
  data-wasm-opt="2"/>`, `#the_canvas_id` canvas, and a loading
  indicator the WASM `main` removes on successful start.
- **C.1** (2026-05-19) — Updated root `README.md`: documents the
  workspace, separates the CLI-only install (`cargo install --path
  .`) from the full-GUI install (`cargo install --path gui`),
  mentions Trunk for the WASM build.

### Test Phase

- **D.1** (2026-05-19) — `cargo build -p oovra` exit 0 (CLI unaffected
  by the workspace stanza).
- **D.2** (2026-05-19) — `cargo test -p oovra` 57/57 PASS (30 + 4 + 23).
- **Test Phase fix** (2026-05-19) — eframe 0.34.2 trait drift caught
  during the first `oovra-gui` build: the `App` trait required `ui`
  (not `update`). Patched `gui/src/app.rs` to implement
  `App::ui(&mut self, ui, frame)` and dropped the `CentralPanel`
  wrapper. Root cause + lesson recorded in `test-report.md` §3.
- **D.3** (2026-05-19) — `cargo test -p oovra-gui` (debug) finished
  in 44.13s. Smoke test `default_app_is_constructible_and_oovra_lib_is_reachable`
  PASS — proves the gui crate links the workspace's `oovra` lib and
  exercises its API at test time.
- **D.4** (2026-05-19) — `cargo build --target wasm32-unknown-unknown
  -p oovra-gui` finished in 1m 06s, exit 0. WASM dep tree resolves;
  shared code paths are free of `std::fs` / `std::net` leaks.
- **D.5** (2026-05-19) — `cargo run -p oovra-gui` launched the
  native window. PID 58088, window title "oovra-gui". User-confirmed
  visual content: bootstrap title, `oovra v0.2.0` link line, and the
  live `is_kebab_case` / `slugify` probe rendering on the default
  `"My Draft"` value as `false` / `Some("my-draft")`. E-1 PASS.
- **Sprint s0 close** (2026-05-19) — `unit-tests.md`,
  `integration-tests.md`, `e2e-tests.md`, and `test-report.md`
  authored in `sprints/s0/sprint-tests/`. Acceptance criteria all
  satisfied. Two logical changes (`create` redesign + s0 bootstrap)
  sit uncommitted, awaiting user authorization.
