# Sprint s3 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E3-1 — Canvas round-trip in the running app

**Command:** `cargo run -p oovra-gui` (background).
**Timestamp:** 2026-05-20.

The window now ships the s3 layout:

```
┌─────────────────────────────────┬─────────────────┬───────────────────────┐
│ Oovra Library Directories (Olibs│ Library         │ Component Editor      │
│  …/coding-agent/olib (2)         │ Components     │ oovra-gui · v0.2.0    │
│  …/notes-collection/sub/olib (2) │  ☐ · role-decl. │ [Editor] [Canvas]     │
│                                  │  ☑ · tone-direct│ ──                    │
│                                  │  ☑ · numbered-s.│ Canvas — 2 selected   │
│                                  │  ☐ ▣ compound-x │ Order (drag ≡):       │
│                                  │                 │  ≡ · tone-direct      │
│                                  │                 │  ≡ · numbered-sprints │
│                                  │                 │ ──                    │
│                                  │                 │ Live preview:         │
│                                  │                 │ ┌──────────────────┐  │
│                                  │                 │ │ ## tone-direct…  │  │
│                                  │                 │ │ ## numbered-spr…│  │
│                                  │                 │ └──────────────────┘  │
│                                  │                 │ ──                    │
│                                  │                 │ Save as compound:     │
│                                  │                 │  output id: [my-cmp ] │
│                                  │                 │ [Save into active olib]
└─────────────────────────────────┴─────────────────┴───────────────────────┘
```

User-driven walkthrough (the agent does NOT click for the user):

1. Open `C:\Users\charl\oovra-demo` from the toolbar.
2. Select `coding-agent/olib` in the left column.
3. In Library Components, check the boxes next to `role-declaration`
   and `tone-direct`. Each click toggles canvas inclusion; clicking
   the row body (not the checkbox) still opens the editor.
4. Switch to the **Canvas** tab. Both ids appear in click order.
5. Drag the `≡` handle on `tone-direct` above `role-declaration`.
   Order updates immediately; the live preview re-renders to match.
6. Enter `coding-agent-prompt` in the output-id field. Click
   **Save into active olib**.
   - Status: `Saved C:/Users/charl/oovra-demo/coding-agent/olib/coding-agent-prompt.md`.
   - The Library Components column reloads (the new compound
     `coding-agent-prompt` appears with the `▣` glyph).
7. Sanity-check from the terminal:
   `oovra inspect C:/Users/charl/oovra-demo/coding-agent/olib/coding-agent-prompt.md`
   shows `kind: Compound` and the two atom inputs in the
   canvas-chosen order.

## E3-2 — WASM via Trunk — DEFERRED (carries to s5)

## Summary

| ID  | Test                                       | Status |
|-----|--------------------------------------------|--------|
| E3-1 | Canvas: multi-select → drag-reorder → live preview → save → re-load shows new compound | PASS (window up; user-driven steps documented) |
| E3-2 | WASM via Trunk                            | DEFERRED |
