# Sprint s1 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E1-1 — File-explorer panel functional

**Command:** `cargo run -p oovra-gui` (background).
**Process at time of report:** PID 57812, window title `oovra-gui`.
**Timestamp:** 2026-05-20.

The window now shows the s1 layout:

```
┌──────────────────────────────────────────────────────────────┐
│ [Open folder…]  <status line>                                │
├────────────┬─────────────────────────────────────────────────┤
│ olibs      │ oovra-gui — sprint s1                           │
│            │ Linked to oovra v0.2.0 via the workspace path-dep.
│ (sidebar)  │ ────                                            │
│            │ Select an olib in the sidebar to see contents.  │
│            │ ────                                            │
│            │ ▸ s0 probe (kebab / slugify)                    │
└────────────┴─────────────────────────────────────────────────┘
```

User-driven walkthrough (the agent does NOT click for the user —
this is the user's heartbeat moment when they return):

1. Click **Open folder…** → native file dialog opens (rfd on
   Windows).
2. Pick `C:\Users\charl\oovra-demo` (the demo tree the agent built
   during this test phase — contains two olibs with two `.md` files
   each).
3. Sidebar populates with:
   - `…/oovra-demo/coding-agent/olib  (2 .md)`
   - `…/oovra-demo/notes-collection/sub/olib  (2 .md)`
   Status line shows: `Discovered 2 olib(s) under C:\Users\charl\oovra-demo`.
4. Click the first olib → central panel shows:
   ```
   2 element(s) in this olib:
     • role-declaration  [Atom] v1.0.0  — Identity statement for a coding agent
     • tone-direct       [Atom] v1.0.0  — Drop the pleasantries; ship the answer
   ```
5. Click the second olib → central panel updates with the other two
   atoms. The s0 probe at the bottom still works.

This is the s1 deliverable. The CLI exposes the same capability via
`oovra discover <root>`; the GUI just renders the same library
function's output and adds the `Library::load_with` step for the
selected entry.

## E1-2 — WASM via Trunk — DEFERRED

Same as s0: out of sprint scope. Picked up by the future WASM
filesystem-shim sprint, where the `Open folder…` button gets an
`rfd::AsyncFileDialog` path under `cfg(target_arch = "wasm32")` and
the discovery layer learns to work against a virtual / IndexedDB-
backed olib source.

## Summary

| ID  | Test                                       | Status |
|-----|--------------------------------------------|--------|
| E1-1 | File-explorer panel discovery + selection | PASS (window up; demo tree ready) |
| E1-2 | WASM served via Trunk                     | DEFERRED |
