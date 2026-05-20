# Sprint s2 — Test Report

**Date:** 2026-05-20
**Sprint:** s2 — Embedded oovra-particle editor + `oovra inspect`
**Verdict:** **PASS** — sprint s2 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 3 (U2-1…U2-3) + 3 carried s1 | 6 | 0 | 0 |
| Integration | 5 (I2-1…I2-5)                | 5 | 0 | 1 (CI) |
| End-to-end  | 1 (E2-1)                     | 1 | 0 | 1 (E2-2 WASM via Trunk) |

Detailed records: [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md),
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria (from build-plan.md §4)

- [x] CLI `oovra inspect <file>` (human + json). (I2-4)
- [x] GUI 3-column layout, click atom → editor, edit → Save → file
      on disk reflects the edit. Compound shows the "not editable
      yet" message. (E2-1, U2-1, U2-2)
- [x] 64 oovra tests + 6 oovra-gui tests, wasm32 build clean. (I2-1, I2-2, I2-3)
- [x] Visual heartbeat captured (window up, layout per spec).

## 3. Issues found and root-caused this sprint

### Issue 1 — `AtomEntry` fields `version` / `meta` flagged dead

**Symptom:** clippy warned that two fields of the `AtomEntry`
helper struct in `app.rs` were never read.

**Surface fix:** Removed `version` and `meta` from `AtomEntry`. The
atoms-list rendering only needs `id`, `kind`, and `path`; the
detailed metadata is shown when the atom is selected (in the
editor panel).

**Underlying cause:** I copied the s1 atom-row layout (id + kind +
version + meta) when planning, then simplified the atoms list to
just `· id` when the editor took over the role of showing details.
The struct was over-fitted to the abandoned UI.

**Disposition:** struct now matches actual usage; clippy clean.

## 4. Deferred items

- **CI / `gh` push verification** — same posture as s0 / s1.
- **E2-2 — WASM via Trunk** — out of s2 scope.
- **egui Panel-alias migration** — still tracked, still deferred.

## 5. Sprint close

Sprint s2 is **complete and locked**. The window is left running
for visual review. The CLI is reinstalled with `oovra inspect` on
PATH. Demo tree at `C:\Users\charl\oovra-demo` is still around for
the user to point the GUI at.

The next pickup is **roadmap determination** per the user's
instruction — picking sprint s3 from the original goals
(autocompose canvas, diff/versioning, WASM file system, etc.).
That selection happens in `sprints/roadmap.md` (not under any
single sprint), after s2 commits.
