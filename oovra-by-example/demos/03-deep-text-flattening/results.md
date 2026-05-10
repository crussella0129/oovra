# Demo 3 — Results

## Composition tree

```
senior-pair-deep (order 2, body_level 2)
├── agent-foundations (order 1, body_level 1)
│   ├── role-statement (order 0)
│   ├── safety-fence (order 0)
│   └── tone-discipline (order 0)
└── output-discipline (order 1, body_level 1)
    ├── format-rules (order 0)
    ├── examples-discipline (order 0)
    └── file-citation-rules (order 0)
```

## Delimiter counts in the order-2 file

| Level | Open count | Close count | Tildes |
|---|---|---|---|
| 2 (outer) | 2 | 2 | 3 |
| 1 (inner, inside the 2 order-1 sub-elements) | 6 | 6 | 2 |

## Prose-output section count

Prose output has **6 H2 sections**. Library contains **6 order-0 atomics**. Equal — meaning every atomic leaf was emitted, and no intermediate-level scaffolding leaked into the output.

## File sizes

| File | Size | Order |
|---|---|---|
| `role-statement.md` | 297 bytes | 0 |
| `safety-fence.md` | 333 bytes | 0 |
| `tone-discipline.md` | 264 bytes | 0 |
| `format-rules.md` | 326 bytes | 0 |
| `examples-discipline.md` | 275 bytes | 0 |
| `file-citation-rules.md` | 222 bytes | 0 |
| `agent-foundations.md` | 1309 bytes | 1 |
| `output-discipline.md` | 1256 bytes | 1 |
| `senior-pair-deep.md` | 2950 bytes | 2 |

## Conclusion

Order-2 file (5,148 bytes, with all metadata and delimiters) flattens via `--text` to ~1,000 bytes of clean prose. The 17 `id = ` references in the on-disk form (each level retaining its full metadata) collapse to 6 H2 section headers in the prose form (one per leaf). Compositional depth is invisible in the prose — exactly what you want for a paste-into-LLM workflow.
