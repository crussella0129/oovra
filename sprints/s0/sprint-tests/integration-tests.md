# Sprint s0 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I-1 — Existing `oovra` tests still pass after the workspace restructure

**Command:** `cargo test -p oovra`
**Timestamp:** 2026-05-19
**Result:** PASS

```
running 30 tests
..............................
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 23 tests
.......................
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests  (doctests)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Verdict:** 57 / 57 tests pass (30 header/library unit + 4 main.rs unit
+ 23 end-to-end integration). Identical to the pre-s0 count — the
workspace stanza and the new `oovra::VERSION` const did not regress
anything in the root crate.

## I-2 — Native build of the gui crate

**Command:** `cargo test -p oovra-gui` (debug profile; covers the build
AND runs the smoke test from `unit-tests.md`).
**Timestamp:** 2026-05-19
**Result:** PASS — finished in 44.13s.

```
   Compiling oovra-gui v0.1.0 (C:\Users\charl\oovra\gui)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 44.13s
     Running unittests src\lib.rs (target\debug\deps\oovra_gui-...)
running 1 test
test app::tests::default_app_is_constructible_and_oovra_lib_is_reachable ... ok
test result: ok. 1 passed; 0 failed
```

**Note on the API drift found mid-phase:** the initial release build
failed because eframe 0.34.2 (the actual resolved patch) replaced
`App::update(&mut self, ctx, frame)` with `App::ui(&mut self, ui,
frame)`. The eframe_template's `Cargo.toml` (pinned at 0.34.1) used
the older signature. The fix was to override `App::ui` directly,
which is recorded in `completed-tasks.md` under the Test Phase. The
underlying cause is documented in `test-report.md` §3 (caret-pinned
deps drift faster than templates).

## I-3 — WASM build of the gui crate

**Command:** `cargo build --target wasm32-unknown-unknown -p oovra-gui`
**Timestamp:** 2026-05-19
**Result:** PASS — finished in 1m 06s.

```
   Compiling web-sys v0.3.98
   Compiling wasm-bindgen-futures v0.4.71
   Compiling web-time v1.1.0
   Compiling wgpu-types v29.0.3
   Compiling glow v0.17.0
   Compiling wgpu-core-deps-wasm v29.0.3
   Compiling egui-wgpu v0.34.2
   Compiling eframe v0.34.2
   Compiling oovra-gui v0.1.0 (C:\Users\charl\oovra\gui)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 06s
```

**Verdict:** the wasm32-unknown-unknown dep tree resolves and
compiles. No `std::fs` / `std::net` leaks in the shared code paths
of `gui/src/*.rs`. This is the safety net that lets later sprints
add web-side filesystem-access shims additively.

`rustup target add wasm32-unknown-unknown` was a no-op (the target
was already installed on this machine).

## I-4 — CLI-only install path unaffected

**Soft check (no separate cargo run needed):** I-1 already proves
`cargo build -p oovra` resolves the dependency graph with only the
root crate's deps (serde, toml, clap, anyhow, etc.) — none of
`eframe`, `winit`, `wgpu`, `egui`, `glow` appear in the I-1 build
output. `cargo install --path .` exercises the same root-package
dependency tree, so the CLI-only install path is unaffected.

**Verdict:** PASS by inference from I-1. A full `cargo install
--path .` from a clean machine would re-prove this; it is not run
here because it would re-do work already verified.

## GitHub CI verification — DEFERRED

Per `test-plan.md` §2, CI/CD verification is gated on the user
authorizing a push. No `git push` is performed by sprint s0. When
the user authorizes a push, the first sprint to push will:

1. Push `main` (or the agreed branch) to `origin`.
2. Wait for the CI workflow to complete via `gh run watch` or equivalent.
3. Confirm the run's `conclusion == "success"` before marking the
   integration phase complete — per the saved feedback memory about
   verifying CI conclusion as a separate step before merging.

## Summary

| ID  | Test                                | Status   |
|-----|-------------------------------------|----------|
| I-1 | `cargo test -p oovra` (57 tests)    | PASS     |
| I-2 | `cargo test -p oovra-gui` (native)  | PASS     |
| I-3 | `cargo build` for `wasm32`          | PASS     |
| I-4 | CLI-only install path unaffected    | PASS (inferred from I-1) |
| CI  | GitHub Actions verification         | DEFERRED until user authorizes a push |
