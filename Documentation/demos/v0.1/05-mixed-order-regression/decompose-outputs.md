# Demo 5 — Decompose Outputs

## One-level decompose

```
[1mDecompose[0m [36mmixed-input-prompt[39m ([2m1.0.0[0m) — order 1, 3 immediate input(s)
  - [36mfoundation[39m [2m@ 1.0.0[0m (order 1) — Foundation
  - [36mtone-discipline[39m [2m@ 1.0.0[0m (order 0) — Tone Discipline
  - [36mformat-rules[39m [2m@ 1.0.0[0m (order 0) — Format Rules
```

## --full decompose folder tree

```
Documentation/demos/05-mixed-order-regression/recovered
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/format-rules.md
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/foundation
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/foundation/foundation.md
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/foundation/role-statement.md
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/foundation/safety-fence.md
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/mixed-input-prompt.md
Documentation/demos/05-mixed-order-regression/recovered/mixed-input-prompt/tone-discipline.md
```

## Byte-equality of recovered leaves vs originals

| Leaf | Result |
|---|---|
| `foundation/role-statement.md` | BYTE-IDENTICAL |
| `foundation/safety-fence.md` | BYTE-IDENTICAL |
| `tone-discipline.md` | BYTE-IDENTICAL |
| `format-rules.md` | BYTE-IDENTICAL |
