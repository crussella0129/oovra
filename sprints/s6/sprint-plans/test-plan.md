# Sprint s6 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-20)

s6 is a refactor (API rename), so it adds **no new tests**. Its
correctness is established by:

1. **Clippy is clean** — the deprecation warnings that the
   `#[allow(deprecated)]` was suppressing no longer exist, so the
   allow could be removed and `cargo clippy --workspace
   --all-targets` still passes with zero warnings. This is the
   load-bearing signal that the migration is complete (any
   remaining deprecated alias would re-trigger a warning).
2. **No regressions** — `cargo test --workspace` still reports the
   same 104 tests passing (58 lib unit + 4 main + 26 integ + 16
   gui), and `cargo build --target wasm32-unknown-unknown
   -p oovra-gui` still succeeds.
3. **Cross-platform parity** — WSL Ubuntu `cargo test -p oovra`
   passes with the same numbers.
4. **Visual identity** — `cargo run -p oovra-gui` shows the same
   three-column layout (toolbar / olibs / components / editor /
   footer probe) as before; default panel widths preserved
   (`default_size` for left panels == old `default_width`).

## CI — DEFERRED (same posture as prior sprints)
