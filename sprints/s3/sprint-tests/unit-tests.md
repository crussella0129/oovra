# Sprint s3 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## Library — `oovra`

No new library code → no new library tests. `render::render_text`,
`render::compose`, `oovra::write` and `Library::load_with` remain
exercised through the 24 integration tests in
`tests/end_to_end.rs`.

## GUI — `oovra-gui::canvas`

Four `#[test]`s in `gui/src/canvas.rs::tests`.

| ID  | Test                                              | Status |
|-----|---------------------------------------------------|--------|
| U3-1 | `canvas_toggle_is_idempotent`                    | PASS |
| U3-2 | `canvas_live_preview_matches_render_text`        | PASS |
| U3-3 | `canvas_save_as_compound_round_trips`            | PASS |
| U3-4 | `canvas_live_preview_errors_on_missing_id`       | PASS |

**U3-1:** toggle is its own inverse (empty → toggle a → [a] →
toggle a → empty); removing a middle entry preserves the order of
the others.

**U3-2:** `canvas.live_preview(&lib)` returns byte-identical output
to a direct `oovra::render::render_text(&[lib.get("tone"),
lib.get("role")])` call. The canvas is a thin wrapper.

**U3-3:** `canvas.save_as_compound(&lib, &olib)` writes a parseable
file. After save, `oovra::parse_file` confirms `kind = Compound`,
`composed_of` lists `[role, tone]` in the canvas-chosen order, and
`body_level == 1` (atoms in → level 1).

**U3-4:** A canvas with an id not in the library surfaces
`OovraError::ElementNotFound { id: "ghost" }` from `live_preview`
rather than panicking or silently dropping.

## GUI — `oovra-gui::app` + `oovra-gui::editor`

Six carry-forward tests from s1 + s2. All still PASS:

- `default_app_is_constructible_and_oovra_lib_is_reachable`
- `short_path_truncates_with_ellipsis`
- `short_path_short_paths_are_not_prefixed`
- `editor_round_trips_an_atom`
- `editor_refuses_to_open_a_compound`
- `editor_rejects_invalid_semver_on_save`

## Grand total

`cargo test -p oovra-gui` reports **10 tests pass**, 0 failed.
Timestamp: 2026-05-20.
