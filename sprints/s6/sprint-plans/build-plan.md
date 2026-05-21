# Sprint s6 — Build Plan

**Finalized — DO NOT EDIT** (2026-05-20)

Migrate `oovra-gui` off the deprecated egui panel aliases. Pure
API-currency; no behavior change. Source:
[`../sprint-research/research-report.md`](../sprint-research/research-report.md).

## Tasks

1. `gui/src/app.rs`:
   - `egui::TopBottomPanel::top("toolbar")` → `egui::Panel::top("toolbar")`.
   - `egui::TopBottomPanel::bottom("footer")` → `egui::Panel::bottom("footer")`.
   - `egui::SidePanel::left("olibs").default_width(280.0)` →
     `egui::Panel::left("olibs").default_size(280.0)`.
   - `egui::SidePanel::left("components").default_width(260.0)` →
     `egui::Panel::left("components").default_size(260.0)`.
   - Remove `#[allow(deprecated)]` on `fn ui` and its trailing
     comment.
2. `CentralPanel::default()` stays as-is (not deprecated).

## Verification (no new tests; this is a refactor)

- `cargo clippy --workspace --all-targets` → 0 warnings (the
  deprecation warnings that motivated the allow are now gone).
- `cargo test --workspace` → existing counts unchanged (104).
- `cargo build --target wasm32-unknown-unknown -p oovra-gui` → PASS.
- WSL Ubuntu `cargo test -p oovra` → PASS, parity with Windows.
- `cargo run -p oovra-gui` → the three-column layout is visually
  identical to pre-migration.

## Out of scope

Everything else. This sprint only swaps deprecated APIs.
