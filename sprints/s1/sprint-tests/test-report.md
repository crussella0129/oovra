# Sprint s1 — Test Report

**Date:** 2026-05-19 → 2026-05-20
**Sprint:** s1 — Olib autodiscovery + file explorer panel
**Verdict:** **PASS** — sprint s1 complete; all acceptance criteria met.

## 1. Result matrix

| Category    | Tests run | Passed | Failed | Deferred |
|-------------|----------:|-------:|-------:|---------:|
| Unit        | 9 (U1-1…U1-9)   | 9 | 0 | 0 |
| Integration | 6 (I1-1…I1-6)   | 6 | 0 | 1 (CI) |
| End-to-end  | 1 (E1-1)        | 1 | 0 | 1 (E1-2: WASM via Trunk) |

Detailed records: [`unit-tests.md`](./unit-tests.md),
[`integration-tests.md`](./integration-tests.md),
[`e2e-tests.md`](./e2e-tests.md).

## 2. Acceptance criteria (from build-plan.md §4)

- [x] Library: `discover()` returns the expected list; respects
      `max_depth`; non-existent root errors cleanly. (U1-1…U1-6, I1-1)
- [x] CLI: `oovra discover .` prints a human summary; `--format json`
      prints valid JSON. (I1-5)
- [x] GUI: **Open folder…** pops a system file picker; picking a
      folder populates the olib sidebar; selecting one loads and
      shows its atoms with id and meta. (E1-1)
- [x] Existing test counts still pass — 64 oovra (57 prior + 6 new
      lib unit + 1 new integration), 3 oovra-gui (1 prior + 2 new).
- [x] `cargo build --target wasm32-unknown-unknown -p oovra-gui`
      still succeeds. (I1-4)

## 3. Issues found and root-caused this sprint

### Issue 1 — egui 0.34.2 panel-alias deprecations

**Symptom:** `cargo clippy` flagged three deprecations on the gui
crate: `egui::TopBottomPanel`, `egui::SidePanel`, and
`Panel::default_width`.

**Surface fix:** `#[allow(deprecated)]` on the `App::ui` impl with a
TODO note. The aliases still work; the deprecations are migrations
to a unified `Panel<Side>` generic and a renamed `default_size`.

**Underlying cause:** same shape as s0's `App::update` → `App::ui`
drift. egui's 0.34 series is consolidating its panel API. Chasing
the new generic Panel API mid-sprint risks scope creep; deferring
it as a discrete migration step in a later sprint keeps the
discovery feature lean.

**Disposition:** logged for a future "egui 0.34.x API tidy" sprint
or rolled into the WASM-features sprint.

### Issue 2 — `open_folder` flagged dead on wasm32

**Symptom:** wasm32 build warned `open_folder` was unused — its only
caller is the `rfd::FileDialog` path, gated to native.

**Surface fix:** `#[cfg_attr(target_arch = "wasm32", allow(dead_code))]`
on the method.

**Underlying cause:** the WASM file-picking story is deferred; the
function legitimately has no callers on that target until the WASM
filesystem-shim sprint wires `rfd::AsyncFileDialog`.

**Disposition:** the cfg-attr is the correct interim — explicit
about why the warning is suppressed.

## 4. Deferred items

- **CI / `gh` push verification.** Gated on the user authorizing a
  push.
- **E1-2 WASM via Trunk.** Out of s1 scope. Returns when the WASM
  filesystem shim sprint lands.
- **egui Panel-alias migration.** Tracked under Issue 1.

## 5. Sprint close

Sprint s1 is **complete and locked**. The window is left running
for visual review. Per [`/CLAUDE.md`](../../../CLAUDE.md) the next
sprint (s2 — embedded editor) rolls immediately; no checkpoint
needed unless an architectural fork or a new visual heartbeat
surfaces.

## 6. Demo tree for the user's visual review

`C:\Users\charl\oovra-demo\` — two olibs with two atoms each:

```
oovra-demo\
├── coding-agent\olib\{role-declaration.md, tone-direct.md}
└── notes-collection\sub\olib\{numbered-sprints.md, research-pattern.md}
```

Point the running GUI at `C:\Users\charl\oovra-demo\` to see
discovery + selection live. `rm -rf` it afterward (or keep it —
it's not in any repo).
