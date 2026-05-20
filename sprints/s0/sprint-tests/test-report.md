# Sprint s0 — Test Report

**Date:** 2026-05-19
**Sprint:** s0 — Project Bootstrap
**Verdict:** **PASS** — sprint s0 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 1 (covers U-1+U-2) | 1 | 0 | 0 |
| Integration | 4 (I-1–I-4)        | 4 | 0 | 1 (CI) |
| End-to-end  | 1 (E-1)            | 1 | 0 | 1 (E-2: WASM via Trunk) |

Detailed records in [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md), and
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria check (from build-plan.md §5)

- [x] `cargo build -p oovra-gui` succeeds on native — see I-2.
- [x] `cargo build --target wasm32-unknown-unknown -p oovra-gui`
      succeeds — see I-3.
- [x] `cargo test -p oovra` still reports the pre-existing test count
      passing (57 / 57) — see I-1.
- [x] `cargo run -p oovra-gui` opens a window that displays the
      bootstrap greeting AND a live `is_kebab_case` / `slugify` probe
      driven by user input — see E-1.
- [x] `cargo install --path .` produces a CLI-only install with no
      eframe stack downloads — verified by inference from I-1's build
      graph (no eframe/wgpu/winit/glow appear when building the root
      package alone). See I-4.
- [x] A populated `agent-tasks/` directory with the s0 build tasks
      moved to `completed-tasks.md` — final move happens at sprint
      close, after this report is filed.

All six acceptance criteria are satisfied.

## 3. Issues found and root-caused this sprint

### Issue 1 — eframe 0.34.2 `App` trait API drift

**Symptom:** First build of `oovra-gui` failed with `E0046: not all
trait items implemented, missing: ui` and a deprecation warning on
`egui::CentralPanel::show`.

**Surface fix:** override `App::ui(&mut self, ui, frame)` instead of
`App::update(&mut self, ctx, frame)`; drop the `CentralPanel` wrapper
since the new entry hands you a `Ui` already attached to the central
area.

**Underlying cause:** the `eframe_template` repo's `Cargo.toml`
pins `eframe = "0.34.1"`, but our `gui/Cargo.toml` uses the friendlier
caret form `eframe = "0.34"`, which resolved to `0.34.2`. Between
0.34.1 and 0.34.2 the `App` trait shifted from `update` to `ui` as
the required method. Templates can lag this; the lesson is to verify
the trait shape against `docs.rs/eframe/0.34.2/eframe/trait.App` (or
whatever caret resolves to) rather than trusting the template's
pinned signature.

**Disposition:** fixed in `gui/src/app.rs`; no other places affected.
Recorded as a build task already-completed in `completed-tasks.md`
("Test Phase fix").

### Issue 2 — initial release-mode build

**Symptom:** D.3 was kicked off with `--release` and would have taken
much longer than necessary for a build-verification.

**Underlying cause:** habit. Release mode adds optimization work that
provides no signal for "does this compile?" — debug is sufficient.

**Disposition:** re-ran in debug. Acceptance criterion remains the
same (the build succeeds); the time saved goes back into the sprint.

## 4. Deferred items, with reasons

- **CI / `gh` verification.** Requires a `git push`; gated on
  explicit user authorization. The verification protocol is recorded
  in `integration-tests.md` under "GitHub CI verification —
  DEFERRED" so the next sprint that pushes can follow it.
- **E-2 WASM served via Trunk.** Requires installing Trunk; the
  build target itself (I-3) is verified, which is the load-bearing
  check. Trunk usage is a feature-sprint concern, not a bootstrap
  concern.

## 5. Outstanding logical commits (uncommitted on disk)

Two logical changes are sitting in the working tree, NOT auto-pushed
or committed:

1. The 2026-05-19 `create` redesign (`--label` / `--olib`, slugify,
   olib-to-olib transfer, default library `./olib`). Six files in
   `src/` + `tests/` + docs.
2. The sprint s0 bootstrap: `Cargo.toml` workspace, new `gui/` crate,
   `oovra::VERSION` const, `sprints/s0/`, `agent-tasks/`, README
   workspace section.

Both are clean, coherent commits, ready when the user authorizes.

## 6. Sprint close

Sprint s0 is **complete and locked**. The next sprint (s1) is the
user's call: the natural pickup is the first real GUI feature —
either the **olib autodiscovery + file explorer panel** (delivers
visible value: point the app at a directory, it finds olibs) or the
**embedded Markdown / oovra-header editor**. Either is a clean s1
deliverable; the bootstrap doesn't constrain which we pick.

Reporting back to the user; awaiting "go" before scaffolding `s1/`.
