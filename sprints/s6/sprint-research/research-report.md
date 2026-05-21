# Sprint s6 — Research Report

**Date:** 2026-05-20
**Sprint goal:** Migrate `oovra-gui` off egui's deprecated panel
type-aliases onto the unified `Panel` API and remove the
`#[allow(deprecated)]` that's been riding on `App::ui` since s2.

This is a pure tech-debt / API-currency sprint — no behavior or
layout change intended.

## 1. The deprecation, confirmed against docs.rs/egui/0.34.2

egui 0.34 consolidated the four panel kinds into one `Panel` type
covering all sides. The old aliases and width/height methods are
deprecated:

| Deprecated | Replacement |
|---|---|
| `egui::TopBottomPanel::top(id)` | `egui::Panel::top(id)` |
| `egui::TopBottomPanel::bottom(id)` | `egui::Panel::bottom(id)` |
| `egui::SidePanel::left(id)` | `egui::Panel::left(id)` |
| `egui::SidePanel::right(id)` | `egui::Panel::right(id)` |
| `.default_width(f32)` / `.default_height(f32)` | `.default_size(f32)` |
| `.min_width` / `.max_width` (and height) | `.min_size` / `.max_size` |

`Panel` is "a panel that covers an entire side (left, right, top or
bottom) of a Ui or screen." `default_size` is orientation-aware — it
maps to width for left/right panels and height for top/bottom. The
`.resizable(bool)`, `.show_inside(ui, …)`, and `CentralPanel`
surfaces are unchanged.

Sources:
- [egui::containers::panel docs](https://docs.rs/egui/0.34.2/egui/containers/panel/index.html)
- [egui::containers::panel::Panel struct docs](https://docs.rs/egui/0.34.2/egui/containers/panel/struct.Panel.html)
  (confirms `default_size` / `min_size` / `max_size` / `exact_size`
  as the current methods; the width/height variants are the
  deprecated ones).

## 2. Sites in oovra-gui

Five touch points in `gui/src/app.rs` (from grep):
- `TopBottomPanel::top("toolbar")`
- `TopBottomPanel::bottom("footer")`
- `SidePanel::left("olibs").default_width(280.0)`
- `SidePanel::left("components").default_width(260.0)`
- `#[allow(deprecated)]` on `fn ui`

`CentralPanel::default()` is not deprecated and stays.

## 3. Risk

Minimal. The `Panel` API is a drop-in for the aliases — same
builder methods, same `show_inside`. The only rename with a
semantic nuance is `default_width → default_size` (orientation
aware), which is exactly right for our left panels. Verification
is "existing 104 tests still pass + clippy clean (no deprecation
warnings) + the window's three-column layout looks identical."
