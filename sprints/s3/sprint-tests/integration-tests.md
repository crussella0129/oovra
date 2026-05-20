# Sprint s3 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I3-1 — `cargo test -p oovra` regression

24 integration tests + lib unit + main unit = unchanged from s2.
The save-as-compound round-trip in U3-3 exercises `render::compose`,
`Library::load_with`, and `oovra::write` end-to-end against a real
on-disk olib. **PASS**.

## I3-2 — `cargo test -p oovra-gui`

**10 tests pass** (3 app + 3 editor + 4 canvas — s2's 6 plus the 4
new canvas tests). **PASS**.

## I3-3 — `cargo build --target wasm32-unknown-unknown -p oovra-gui`

**12.12s, exit 0.** egui_dnd 0.15.0 compiles cleanly for wasm32 (it
pulls in `egui_animation` 0.11.0 as a transitive dep — also clean).
The canvas tab works the same on wasm32 *at compile time*; runtime
filesystem ops (save-as-compound's `oovra::write`) will fail in the
browser sandbox until the WASM filesystem shim sprint (s5) lands.
**PASS**.

## I3-4 — workspace clippy clean

After mirroring `#![allow(clippy::result_large_err)]` from the root
crate into `gui/src/lib.rs` (same justification — `OovraError`
carries a large variant; suppressing the lint at the crate root
matches upstream policy), `cargo clippy --workspace --all-targets`
returns 0 warnings, exit 0. **PASS**.

## I3-5 — Ubuntu Linux build + test via WSL

Stable Rust 1.95.0 bootstrapped into the WSL Ubuntu image; the
oovra CLI builds and the full test suite passes Linux-side:

```
$ wsl.exe -- bash -lc 'source $HOME/.cargo/env && \
    cd /mnt/c/Users/charl/oovra && \
    CARGO_TARGET_DIR=/tmp/oovra-linux-target cargo build -p oovra'
  Compiling oovra v0.2.0 (/mnt/c/Users/charl/oovra)
   Finished `dev` profile in 13.70s

$ wsl.exe -- bash -lc 'source $HOME/.cargo/env && \
    cd /mnt/c/Users/charl/oovra && \
    CARGO_TARGET_DIR=/tmp/oovra-linux-target cargo test -p oovra'
running 24 tests ... ok
test result: ok. 24 passed; 0 failed
```

`CARGO_TARGET_DIR=/tmp/oovra-linux-target` keeps the Linux build
artifacts out of `target/`, which the Windows build owns. **PASS.**

The cross-platform requirement and the WSL invocation are now
codified in [`/CLAUDE.md`](../../../CLAUDE.md) (Cross-platform
section) so future sprints don't drift Windows-only.

## CI verification — DEFERRED (same posture as prior sprints)

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| I3-1 | oovra tests                                  | PASS |
| I3-2 | oovra-gui tests (10)                         | PASS |
| I3-3 | wasm32 build of oovra-gui                    | PASS |
| I3-4 | workspace clippy clean                       | PASS |
| I3-5 | Ubuntu Linux build + test via WSL            | PASS |
| CI  | GitHub Actions verification                   | DEFERRED |
