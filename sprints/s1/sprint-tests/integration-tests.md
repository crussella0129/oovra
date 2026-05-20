# Sprint s1 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I1-1 — `discover_finds_two_nested_olibs` (library integration)

**File:** `tests/end_to_end.rs`.

Builds a temp tree:

```
root/
├── proj-a/olib/{intro.md, tone.md}    # shallow olib, 2 .md
├── proj-b/sub/olib/atom.md            # deeper olib, 1 .md
└── notes/scratch.md                   # decoy: dir is NOT olib
```

Asserts `discover(root, None)` returns both olibs in sorted-path
order with `md_count == [2, 1]`. The decoy `notes/` directory must
not appear.

**Result:** PASS. `cargo test -p oovra` reports 24 integration tests
(23 pre-existing + I1-1).

## I1-2 — pre-existing 57 oovra tests still pass

`cargo test -p oovra` — 36 + 4 + 24 = **64 tests pass** (the 57 from
sprint s0 plus the 6 new unit tests in `discovery.rs` and the 1 new
integration test I1-1). **PASS**.

## I1-3 — `oovra-gui` smoke test still passes

`cargo test -p oovra-gui` — 3 unit tests pass (the s0 smoke,
`short_path_truncates_with_ellipsis`, and
`short_path_short_paths_are_not_prefixed`). **PASS**, 15.56s.

## I1-4 — `oovra-gui` wasm32 build still succeeds

`cargo build --target wasm32-unknown-unknown -p oovra-gui` finished
`dev` profile in 9.94s, exit 0. The wasm32 branch of the
`Open folder…` button is a `cfg(target_arch = "wasm32")` stub that
sets a deferred-feature status string; the native rfd call is gated
to `cfg(not(target_arch = "wasm32"))`, so the wasm32 build never
references `rfd`. **PASS**.

## I1-5 — CLI smoke (manual)

- `oovra discover C:/Users/charl/oovra/sprints` — reports `0 olib(s)
  found.` (no `olib/` dir under `sprints/`). **PASS**.
- `oovra discover C:/Users/charl/oovra-demo` (the s1 demo tree built
  during the test phase) — reports two olibs at the right paths
  with the expected `.md` counts:

  ```
  Discover C:/Users/charl/oovra-demo (max depth: unlimited)
    ✓ C:/Users/charl/oovra-demo\coding-agent\olib  (2 .md)
    ✓ C:/Users/charl/oovra-demo\notes-collection\sub\olib  (2 .md)
  2 olib(s) found.
  ```

  **PASS**.

- `oovra discover C:/Users/charl/oovra-demo --format json` returns a
  one-line valid JSON array `[{"path":...,"md_count":2}, ...]`.
  **PASS**.

## I1-6 — workspace clippy

`cargo clippy --workspace --all-targets` returns 0 errors and 0
warnings after the s1 fixes (`#[cfg_attr(target_arch = "wasm32",
allow(dead_code))]` on `open_folder` and `#[allow(deprecated)]` on
the eframe::App::ui method for the egui 0.34.2 panel-alias
deprecations — to be removed in a later migration sprint). **PASS**.

## CI verification — DEFERRED

Same posture as s0: gated on user authorization to push. Local
verification is the load-bearing check.

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| I1-1 | discover_finds_two_nested_olibs (lib)        | PASS |
| I1-2 | 57+ oovra tests still pass                   | PASS (64 tests) |
| I1-3 | oovra-gui smoke still passes                 | PASS |
| I1-4 | wasm32 build of oovra-gui                    | PASS |
| I1-5 | `oovra discover` CLI smoke (human + JSON)    | PASS |
| I1-6 | workspace clippy                             | PASS |
| CI  | GitHub Actions verification                   | DEFERRED |
