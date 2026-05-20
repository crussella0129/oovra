# Sprint s2 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## Library — `oovra`

No new library code in s2 → no new library tests. The save path
(`oovra::write` on a freshly built `PromptElement`) is exercised
through `editor_round_trips_an_atom` (below) and was already covered
by the in-memory validation tests in `src/element.rs`.

## GUI — `oovra-gui::editor`

Three `#[test]`s in `gui/src/editor.rs::tests`.

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| U2-1 | `editor_round_trips_an_atom`                 | PASS   |
| U2-2 | `editor_refuses_to_open_a_compound`          | PASS   |
| U2-3 | `editor_rejects_invalid_semver_on_save`      | PASS   |

**U2-1 — open → edit body → save → re-read.** Writes an atom via
`oovra::create::label_into_olib`, opens it through `Editor::open`,
mutates the body, calls `Editor::save()`, then `oovra::parse_file`s
the result and asserts (a) the new body landed, (b) the id is
unchanged, (c) the dirty flag was cleared.

**U2-2 — compound rejection.** Writes a minimal compound file
directly, passes its path to `Editor::open`, asserts the return
is `OpenResult::CompoundReadOnly` — not `Loaded`, not a panic.

**U2-3 — invalid semver is caught BEFORE writing.** After opening
a valid atom, mutates `editor.version` to `"not-a-version"`, calls
`save()`, asserts the error, then `parse_file`s the on-disk file
and confirms the original `version = "1.0.0"` is intact. The
save's pre-write validation is the load-bearing guarantee.

## GUI — `oovra-gui::app`

s1 tests carry forward unchanged (smoke + the two `short_path`
helpers), all PASS, total 3.

## Grand total

`cargo test -p oovra-gui` reports **6 tests pass** (3 editor + 3
app), 0 failed.

Timestamp: 2026-05-20.
