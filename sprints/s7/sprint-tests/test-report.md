# Sprint s7 — Test Report

**Date:** 2026-05-20
**Sprint:** s7 — Inline header tags in the component tree
**Verdict:** **PASS** — feature complete; no regressions.

## 1. What changed

`gui/src/app.rs`:
- `render_component_node` now reads `version` + `meta` for each
  node (in the same scoped `self.loaded` borrow that fetches the
  cascade leaves).
- New free fn `tag_labels(ui, version, meta)` renders `v<version>`
  and a dimmed, truncated `— <meta>` after the row's label /
  collapsing header. Meta longer than 40 chars is ellipsized.

This delivers the original-vision item "see the tags from the
Oovra header without opening." Display-only — no library, schema,
or recipe change.

## 2. Verification matrix

| Check | Result |
|---|---|
| `cargo clippy --workspace --all-targets` | clean |
| `cargo test --workspace` (Windows) | **104 PASS** (58 + 4 + 26 + 16); unchanged |
| `cargo build wasm32 -p oovra-gui` | PASS |
| WSL Ubuntu `cargo test -p oovra` | parity (88) |
| `cargo run -p oovra-gui` | tree rows show `v<version>  — <meta>` dimmed inline |

No new automated tests — `tag_labels` is a pure display helper that
renders into a `Ui` (no return value to assert on), and the change
touches no logic paths exercised by the suite. Verification is the
unchanged test count + clippy clean + the visual heartbeat.

## 3. Sprint close

Sprint s7 is **complete and locked**. Tree rows now carry their
version and a short meta inline.

Remaining roadmap: WASM filesystem shim + Trunk pipeline, compound
recipe editing, cross-olib compare, syntax-highlighted editor +
Markdown preview, agent-API hardening.
