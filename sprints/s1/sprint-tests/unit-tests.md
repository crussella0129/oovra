# Sprint s1 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## Library — `oovra::discovery`

Six `#[test]`s in `src/discovery.rs::tests` (commit-resident).

| ID  | Test                                              | Status |
|-----|---------------------------------------------------|--------|
| U1-1 | `discover_finds_shallow_olib`                    | PASS |
| U1-2 | `discover_finds_deep_olib`                       | PASS |
| U1-3 | `discover_respects_max_depth`                    | PASS |
| U1-4 | `discover_ignores_non_olib_dirs_and_top_level_md`| PASS |
| U1-5 | `discover_errors_on_missing_root`                | PASS |
| U1-6 | `discover_returns_deterministic_order`           | PASS |

Run: `cargo test -p oovra` — 36 lib unit tests pass (30 prior + 6 new
discovery tests), 4 main.rs unit tests pass, 24 integration tests
pass (23 prior + 1 new — see I1-1 in `integration-tests.md`). 64
oovra tests total, all green.

## GUI — `oovra-gui::app`

| ID  | Test                                                  | Status |
|-----|-------------------------------------------------------|--------|
| U1-7 | `default_app_is_constructible_and_oovra_lib_is_reachable` (s0 smoke, expanded for s1 state) | PASS |
| U1-8 | `short_path_truncates_with_ellipsis`                 | PASS |
| U1-9 | `short_path_short_paths_are_not_prefixed`            | PASS |

Run: `cargo test -p oovra-gui` — 3 unit tests pass in 15.56s.

**Timestamp:** 2026-05-19 / 2026-05-20.

## Notes

- The `OovraApp` smoke test (U1-7) carries forward from s0 and was
  extended for s1 to assert `discovered`, `selected`, `loaded`, and
  `status` initialize empty — so a fresh app shows the "open a
  folder to discover olibs" hint.
- `short_path` is a tiny display helper for the sidebar; testing it
  here keeps it honest as a free function and documents the
  ellipsis-prefix rule.
