# Sprint s3 — Test Report

**Date:** 2026-05-20
**Sprint:** s3 — Live autocompose canvas
**Verdict:** **PASS** — sprint s3 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 4 (U3-1…U3-4) + 6 carried | 10 | 0 | 0 |
| Integration | 5 (I3-1…I3-5)             | 5  | 0 | 1 (CI) |
| End-to-end  | 1 (E3-1)                  | 1  | 0 | 1 (E3-2 WASM via Trunk) |

Detailed records: [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md),
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria (from build-plan.md §4)

- [x] Central panel has Editor / Canvas tabs (visual heartbeat).
- [x] Per-row checkbox in Library Components toggles canvas
      inclusion independently of editor selection.
- [x] Canvas tab shows the selected components as a drag-reorderable
      list, a live preview, and a Save form.
- [x] Save into the active olib produces a parseable compound file
      and the new compound appears in Library Components after
      reload (U3-3 + E3-1).
- [x] All existing tests pass; wasm32 build still clean.

## 3. Issues found and root-caused this sprint

### Issue 1 — `clippy::result_large_err` on `CanvasState::resolve`

**Symptom:** clippy flagged the `Result<Vec<&PromptElement>, OovraError>`
return type because `OovraError` is large (~128 bytes, dominated by
its boxed `toml::de::Error` variant).

**Underlying cause:** the root `oovra` crate already suppresses this
exact lint at its lib.rs root with `#![allow(clippy::result_large_err)]`
and a comment explaining why (lib.rs:12). The gui crate's lib.rs
didn't carry the same allow.

**Surface fix:** added `#![allow(clippy::result_large_err)]` to
`gui/src/lib.rs` with a referencing comment. Same rationale as the
upstream allow; not a behavior change.

**Disposition:** policy is now consistent across the workspace.

## 4. Cross-platform check landed in this sprint

User raised it mid-sprint after noticing the `.exe` references.
WSL Ubuntu's rustup is installed; the oovra CLI builds and
**all 24 integration tests pass on Linux** with
`CARGO_TARGET_DIR=/tmp/oovra-linux-target`. The requirement is
codified in `/CLAUDE.md` so future sprints don't drift Windows-only.
See I3-5 in `integration-tests.md`.

The GUI's Linux build wasn't run this sprint — eframe needs apt
dev packages (`libxcb-render0-dev libxcb-shape0-dev
libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`) and that's a
one-time host setup. It's the next sprint that touches GUI build
config that should land that, not s3.

## 5. Deferred items

- **CI / `gh` push verification** — same posture as prior sprints.
- **E3-2 — WASM via Trunk** — out of s3 scope.
- **egui Panel-alias migration** — still tracked for a discrete
  sprint (roadmap s6).
- **GUI Linux build verification** — requires apt prereqs on the
  WSL Ubuntu image; pick up in the next GUI-touching sprint.

## 6. Sprint close

Sprint s3 is **complete and locked**. The window stays up for the
user's review. Per the roadmap, **s4 — diff / versioning view** is
the natural next pickup. The canvas now provides the
multi-selection + ordering substrate that s4's "compare two
compositions" view will likely lean on.
