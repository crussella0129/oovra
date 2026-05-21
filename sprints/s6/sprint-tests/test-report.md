# Sprint s6 — Test Report

**Date:** 2026-05-20
**Sprint:** s6 — egui Panel-alias migration
**Verdict:** **PASS** — migration complete; no regressions.

## 1. What changed

`gui/src/app.rs`: four panel constructions migrated off deprecated
aliases, one `default_width` → `default_size`, and the
`#[allow(deprecated)]` on `App::ui` removed.

| Before | After |
|---|---|
| `TopBottomPanel::top("toolbar")` | `Panel::top("toolbar")` |
| `TopBottomPanel::bottom("footer")` | `Panel::bottom("footer")` |
| `SidePanel::left("olibs").default_width(280.0)` | `Panel::left("olibs").default_size(280.0)` |
| `SidePanel::left("components").default_width(260.0)` | `Panel::left("components").default_size(260.0)` |
| `#[allow(deprecated)] fn ui` | `fn ui` (allow removed) |

## 2. Verification matrix

| Check | Result |
|---|---|
| `cargo clippy --workspace --all-targets` | **0 warnings** (deprecation gone; allow removed cleanly) |
| `cargo test --workspace` (Windows) | **104 PASS** (58 + 4 + 26 + 16); unchanged |
| `cargo build --target wasm32-unknown-unknown -p oovra-gui` | PASS (11.24s) |
| WSL Ubuntu `cargo test -p oovra` | **88 PASS** (58 + 4 + 26); parity with Windows |
| `cargo run -p oovra-gui` | window up, three-column layout visually identical |

The clippy-clean-without-the-allow result is the proof the
migration is total: had any deprecated alias survived, removing the
`#[allow(deprecated)]` would have re-surfaced a warning and clippy
(run with default deny-nothing but we read warnings) would have
reported it.

## 3. No issues found

Drop-in migration; `default_size` is orientation-aware and maps to
width for the left panels, preserving the 280px / 260px defaults.

## 4. Sprint close

Sprint s6 is **complete and locked**. The gui crate now uses only
non-deprecated egui panel APIs and carries no `#[allow(deprecated)]`.

Remaining roadmap: syntax-highlighted editor + Markdown preview
(was roadmap s7), the WASM filesystem shim + Trunk pipeline (was
roadmap s5, deferred), compound recipe editing, cross-olib compare,
and agent-API hardening.
