# Sprint s1 — Test Plan

**Finalized — DO NOT EDIT** (2026-05-19)

Counterpart to [`build-plan.md`](./build-plan.md). Library coverage
is the load-bearing layer (the CLI and GUI are thin); s1's tests
weight accordingly.

## 1. Unit tests (library)

In `src/discovery.rs`'s `#[cfg(test)] mod tests`:

- **U1-1 — shallow hit.** A directory tree `root/olib/` with one
  `.md` file: `discover(root, None)` returns one
  `DiscoveredOlib { path: root/olib, md_count: 1 }`.
- **U1-2 — deep hit.** A tree `root/a/b/olib/` with two `.md` files:
  `discover(root, None)` finds it; `md_count == 2`.
- **U1-3 — max_depth bounds the walk.** With both shallow and deep
  olibs, `discover(root, Some(1))` returns only the shallow one
  (depth measured from `root`).
- **U1-4 — non-olib dirs ignored.** A tree containing a directory
  literally named `notes` with `.md` files is **not** returned.
  Likewise a top-level `.md` file is not an olib.
- **U1-5 — non-existent root errors.** `discover("does/not/exist", None)`
  returns an `OovraError::FileNotFound` (or equivalent).
- **U1-6 — deterministic ordering.** Results sorted by path so the
  CLI / GUI render in a stable order regardless of walkdir's
  internal order.

## 2. Integration tests

In `tests/end_to_end.rs`:

- **I1-1 — discover_finds_two_nested_olibs.** Build a temp tree with
  two olib dirs at different depths, write a couple of plain `.md`
  files in each, call `oovra::discovery::discover` and assert the
  returned set of paths matches the expected set. (Library-level
  integration; the CLI command is a thin wrapper over this.)
- **I1-2 — pre-existing 57 oovra tests still pass.** Regression
  guard against the discovery module breaking unrelated parts of
  the crate (e.g. lib.rs re-exports).
- **I1-3 — gui smoke test still passes.** `cargo test -p oovra-gui`
  — the gui crate's smoke test (`default_app_is_constructible_…`)
  must still pass after the OovraApp state expansion.
- **I1-4 — wasm32 build of `oovra-gui` still succeeds.**
  `cargo build --target wasm32-unknown-unknown -p oovra-gui`. The
  rfd sync `pick_folder` call must be cfg-gated to native so the
  wasm32 build doesn't fail. Verified by running the wasm32 build.

## 3. End-to-End test

- **E1-1 — visual heartbeat.** `cargo run -p oovra-gui` in the
  background. User-observable behavior:
  - The window now has a top toolbar with an **Open folder…**
    button.
  - Clicking it pops a system file picker.
  - Picking a folder that contains at least one `olib/` populates
    the left sidebar with the discovered olib(s) and their `.md`
    counts.
  - Selecting an olib shows its parsed atoms in the central panel
    with each atom's id and meta visible.
  - The s0 kebab/slugify probe is still rendered (sprint-over-sprint
    visual diff), at the bottom.
  - Closing the window cleanly returns control.

  The test plan does NOT require the test environment to literally
  click through this — the agent confirms by visual inspection
  during the run, and the user will see the same UI when they next
  open the app.

## 4. CLI smoke (manual, in the test report)

- `oovra discover ./sprints` should report no olibs (no `olib/`
  directories under sprints/).
- `oovra discover <tmp-tree-with-2-olibs> --format json` returns
  valid JSON parseable by `jq .`.

These are recorded in `integration-tests.md` under "CLI smoke."

## 5. Logging conventions

Same as s0 — each test goes into `sprint-tests/*.md` with:

- ID (U1-N / I1-N / E1-N).
- Command, framework, or `#[test]` name.
- Pass / fail.
- Verbose output excerpt that demonstrates what was checked.
- Timestamp.

`test-report.md` summarizes pass/fail/deferred per category and
checks off `build-plan.md` §4 acceptance criteria.
