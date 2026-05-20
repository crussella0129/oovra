# Sprint s2 — Integration Tests

Per [`../sprint-plans/test-plan.md`](../sprint-plans/test-plan.md) §2.

## I2-1 — `cargo test -p oovra` regression

64 tests pass (36 lib unit + 4 main unit + 24 integration). Identical
to the s1 close count — s2 added no library code, so no count change
expected. **PASS**.

## I2-2 — `cargo test -p oovra-gui`

6 tests pass (3 s1-prior + 3 new editor tests from U2-1/U2-2/U2-3).
**PASS**.

## I2-3 — `cargo build --target wasm32-unknown-unknown -p oovra-gui`

Builds clean. The editor module compiles on wasm32; runtime use of
`oovra::write` against the browser would fail (no filesystem), but
that is a *runtime* concern handled by the future WASM
filesystem-shim sprint. The build is the safety net. **PASS**.

## I2-4 — `oovra inspect` CLI smoke

Against the s1 demo tree `C:\Users\charl\oovra-demo`:

```
$ oovra inspect C:/Users/charl/oovra-demo/coding-agent/olib/role-declaration.md
Inspect C:/Users/charl/oovra-demo/coding-agent/olib/role-declaration.md
  id        role-declaration
  name      role-declaration
  kind      Atom
  version   1.0.0
  meta      Identity statement for a coding agent
  body      1 line(s), 74 chars

$ oovra inspect ...role-declaration.md --format json
{"body_chars":74,"body_level":null,"body_lines":1,"composed_of":null,
"depth":null,"generated_at":null,"id":"role-declaration","kind":"atom",
"meta":"Identity statement for a coding agent","name":"role-declaration",
"render_mode":null,"version":"1.0.0"}

$ oovra inspect C:/Users/charl/oovra-demo/nope.md
Error: reading C:/Users/charl/oovra-demo/nope.md
Caused by: File not found: C:/Users/charl/oovra-demo/nope.md
```

Human format: PASS. JSON format: valid single-line JSON. Missing
file: clean error, non-zero exit. **PASS**.

## I2-5 — workspace clippy

`cargo clippy --workspace --all-targets` returns exit 0 with no
warnings after the AtomEntry trim (the `version`/`meta` fields were
unused since the editor displays them; removed from the struct).
**PASS**.

## CI verification — DEFERRED (still gated on user authorization to push)

## Summary

| ID  | Test                                          | Status |
|-----|-----------------------------------------------|--------|
| I2-1 | oovra tests (64)                             | PASS |
| I2-2 | oovra-gui tests (6)                          | PASS |
| I2-3 | wasm32 build of oovra-gui                    | PASS |
| I2-4 | `oovra inspect` CLI smoke (human + json + missing) | PASS |
| I2-5 | workspace clippy clean                       | PASS |
| CI  | GitHub Actions verification                   | DEFERRED |
