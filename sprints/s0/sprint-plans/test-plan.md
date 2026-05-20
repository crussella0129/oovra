# Sprint s0 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-19)

Counterpart to [`build-plan.md`](./build-plan.md). The s0 sprint
delivers a bootstrap — there is no domain logic to unit-test yet, so
the bulk of testing is build-time (does the workspace + WASM target
compile?) and integration-time (do existing oovra tests still pass
after the restructure?). End-to-End testing is a single visual check
that the native app actually opens a window with the right content.

## 1. Unit tests

The s0 deliverable adds no new business logic — the gui crate is a
shell that calls into the `oovra` library. Two cheap unit tests
nevertheless earn their keep:

- **U-1: `OovraApp::default()` is constructible.** A `#[test]` in
  `gui/src/app.rs` (or `gui/tests/smoke.rs`) that calls
  `let _ = OovraApp::default();` and ends. This catches any default
  field that requires runtime state (e.g. accidentally a path that
  doesn't exist).
- **U-2: A library call from the gui crate compiles and runs.** The
  same smoke test calls
  `assert!(oovra::header::is_kebab_case("my-id"))`
  — proves the path-dep is wired and the library's surface is
  reachable from the gui crate at test time, not just from `main.rs`.

Both U-1 and U-2 are bundled into one `#[test]` if convenient.

The existing 57 tests in the root crate are *also* unit / integration
tests for the library and CLI; the restructure must not regress them.
That is captured under "Integration tests" below.

## 2. Integration tests

- **I-1: existing oovra tests still pass.** `cargo test -p oovra`
  reports the same total count as before s0 (header 30 unit, main 4
  unit, end_to_end 23 integration — 57 total) and zero failures.
  This is the regression guard for the workspace restructure.
- **I-2: native gui builds.** `cargo build -p oovra-gui` exits 0.
  This is the build verification for the eframe stack on this
  machine.
- **I-3: WASM gui builds.** `rustup target add wasm32-unknown-unknown`
  (idempotent), then `cargo build --target wasm32-unknown-unknown
  -p oovra-gui` exits 0. This verifies that *no shared code path*
  in the gui crate calls into APIs missing on `wasm32-unknown-unknown`
  (`std::fs`, `std::net`, etc.). It is the safety net that makes
  later WASM feature work additive rather than reactive.
- **I-4: CLI-only install path is unaffected.** `cargo install --path
  .` from the repo root completes without downloading the eframe
  dependency tree. Verified by inspecting the install output for
  absence of `eframe` / `winit` / `wgpu` compile lines. Soft check
  (eyeballed by the agent and logged in test-report.md); a hard
  programmatic check is out of scope this sprint.
- **GitHub CI verification — DEFERRED.** The methodology calls for
  CI/CD verification in this phase. We will not push from sprint s0;
  CI verification is deferred to the first sprint where the user
  authorizes a push. The deferral, with reason, is recorded in
  `integration-tests.md`.

## 3. End-to-End tests

- **E-1: the app opens.** `cargo run -p oovra-gui` (run in the
  background) opens a native window titled "oovra-gui — sprint s0
  bootstrap". The user can interact: typing in the kebab probe field
  shows live `is_kebab_case` and `slugify` output, proving the gui
  crate is exercising the real `oovra` library at runtime. Per the
  user's saved preference, the agent launches it so the user gets
  a window to look at; the agent does not push a "please run X"
  request onto them.
- **E-2: WASM app served via Trunk — DEFERRED.** Serving the WASM
  build requires installing `trunk` and starting a dev server. Out
  of s0 scope; documented in `e2e-tests.md` as the gate for a later
  sprint that picks up Trunk and starts adding the file-system-access
  layer for the web target.

## 4. Logging conventions

Each completed test goes into the matching `sprint-tests/*.md`
document with:

- Test ID (U-N / I-N / E-N).
- Exact command run, or test name + framework.
- Pass / fail verdict.
- Verbose output excerpt sufficient to understand what was checked
  (counts, key lines from cargo output, etc.).
- Timestamp of the run.

`test-report.md` is a one-page summary at the end of the phase: total
pass / fail / deferred per category, and the sprint s0 acceptance
criteria from `build-plan.md` §5 checked off.

## 5. Reviewed for correctness and conciseness

This plan covers the s0 deliverable end-to-end. Concise where the
sprint is shallow (unit tests are thin because the bootstrap has
little logic) and explicit where the sprint depends on tooling that
isn't on the box (Trunk, CI).
