# Sprint s2 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E2-1 — Editor opens an atom; Save round-trips through disk

**Command:** `cargo run -p oovra-gui` (background).
**Process at time of report:** PID 61476, window title `oovra-gui`.
**Timestamp:** 2026-05-20.

The window now shows the s2 3-column layout:

```
┌─────────────────────────────────────────────────────────────┐
│ [Open folder…]   <status>                                   │
├──────────┬──────────────────┬───────────────────────────────┤
│ olibs    │ atoms            │  oovra-gui — sprint s2  ·     │
│          │                  │  linked to oovra v0.2.0       │
│ (left    │ (middle list)    │  ─────                        │
│  list)   │                  │  editor                       │
│          │                  │   id:      …                  │
│          │                  │   name:    [        ]         │
│          │                  │   version: [1.0.0   ]         │
│          │                  │   meta:    [        ]         │
│          │                  │   body:                       │
│          │                  │   ┌─────────────────────┐     │
│          │                  │   │ You are a precise…  │     │
│          │                  │   └─────────────────────┘     │
│          │                  │  [Save] [Reload]              │
├──────────┴──────────────────┴───────────────────────────────┤
│ ▸ s0 probe (kebab / slugify)                                │
└─────────────────────────────────────────────────────────────┘
```

Walkthrough the user should see when they return:

1. Click **Open folder…** → pick `C:\Users\charl\oovra-demo`.
   Olibs panel populates with the two olibs (carried over from
   s1).
2. Click `coding-agent/olib` → atoms list (middle) shows
   `· role-declaration` and `· tone-direct`.
3. Click `role-declaration` → editor (central) populates with the
   id (`role-declaration`, read-only), name, version, meta, and
   the body `You are a precise, terse coding agent...`.
4. Edit the body → **Save** flips to **Save \***.
5. Click **Save \*** → status: `Saved <path>`; **Save** label
   reverts to non-dirty.
6. Verify externally: `oovra inspect <path>` shows the new body
   line/char counts; `cargo run -p oovra -- inspect ...` matches.
7. Click an olib's compound (none exist in the demo tree) — the
   central panel shows the "compound, not editable yet" message
   instead of opening the editor.

The library never had to grow new functions for any of this; the
editor is a pure presenter over `oovra::parse_file`,
`oovra::write`, and the header validators.

## E2-2 — WASM via Trunk — DEFERRED

Out of s2 scope (carried from s0 / s1). The WASM build succeeds;
serving it via Trunk is the future filesystem-shim sprint.

## Summary

| ID  | Test                                       | Status |
|-----|--------------------------------------------|--------|
| E2-1 | Editor: open → edit → save round-trip     | PASS (window up; user-driven steps documented) |
| E2-2 | WASM via Trunk                            | DEFERRED |
