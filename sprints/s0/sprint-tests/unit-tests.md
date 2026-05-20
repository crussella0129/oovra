# Sprint s0 — Unit Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §1.

## U-1 + U-2 (combined) — gui-crate smoke test

**Test:** `oovra_gui::app::tests::default_app_is_constructible_and_oovra_lib_is_reachable`

**Source:** [`gui/src/app.rs`](../../../gui/src/app.rs) — the `#[cfg(test)]
mod tests` block.

**What it covers**

- U-1 — `OovraApp::default()` builds without panicking; default state
  has a non-empty `kebab_probe`.
- U-2 — a real `oovra::header::is_kebab_case("my-id")` call succeeds
  from the gui crate (proves the path-dep + library surface are wired
  together), and `oovra::header::slugify("My Draft")` returns
  `Some("my-draft")` (proves the redesigned API is reachable too).

**Result:** PASS

```
running 1 test
test app::tests::default_app_is_constructible_and_oovra_lib_is_reachable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.00s
```

**Command:** `cargo test -p oovra-gui` (debug profile)
**Timestamp:** 2026-05-19
**Verdict:** Both U-1 and U-2 satisfied by a single `#[test]`.

## Summary

| ID  | Test                                                         | Status |
|-----|--------------------------------------------------------------|--------|
| U-1 | `OovraApp::default()` is constructible                       | PASS   |
| U-2 | A library call from the gui crate compiles and runs at test time | PASS |
