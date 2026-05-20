# Sprint s0 — End-to-End Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §3.

## E-1 — Native app opens and exercises the `oovra` library at runtime

**Command:** `cargo run -p oovra-gui` (background)
**Timestamp:** 2026-05-19
**Result:** PASS — visual confirmation from the user.

```
   Compiling oovra-gui v0.1.0 (C:\Users\charl\oovra\gui)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.88s
     Running `target\debug\oovra-gui.exe`
```

A native window opens (process `oovra-gui.exe` PID 58088, window title
`oovra-gui`). User-reported window content, verbatim:

```
oovra-gui — sprint s0 bootstrap
Linked to oovra v0.2.0 via the workspace path-dep.

Live probe — exercising oovra::header at runtime:
filename stem:
  is_kebab_case = false
  slugify       = Some("my-draft")

This shell is the s0 deliverable. File explorer, syntax-highlighted
editor, and the autocompose canvas come in later sprints.
```

**What this proves**

- The native binary launches into an egui window — eframe stack ok
  end-to-end on this machine.
- The `oovra::VERSION` constant resolved to `0.2.0` — the gui crate
  is linking the workspace's `oovra` lib via the path-dep, not a
  stale crates.io version.
- The live probe is executing real `oovra::header::is_kebab_case` and
  `oovra::header::slugify` calls every frame against user input —
  the redesigned API surface is reachable from the GUI at runtime,
  not just at link time.
- The default `kebab_probe` value `"My Draft"` slugifies to
  `my-draft` — the exact mapping unit-tested in `header.rs`'s
  `slugify_specific_mappings` test, now observed at runtime in the
  GUI.

This satisfies E-1 per the test plan. The window is left running for
further interactive use by the user.

## E-2 — WASM app served via Trunk — DEFERRED

Out of s0 scope. Requires installing the Trunk dev server
(`cargo install --locked trunk`) and starting `trunk serve` from
`gui/`. Picked up by the first later sprint that begins adding
filesystem-access shims for the web target — at which point Trunk
becomes part of the dev loop.

## Summary

| ID  | Test                                                  | Status   |
|-----|-------------------------------------------------------|----------|
| E-1 | Native window opens; bootstrap content + live probe render | PASS (user visual sign-off) |
| E-2 | WASM build served via Trunk                           | DEFERRED |
