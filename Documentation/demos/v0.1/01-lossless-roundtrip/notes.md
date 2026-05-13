# Demo 1 — Lossless Round-Trip

## What this demonstrates

The central architectural claim of Oovra: **a composed file is its own self-extracting archive.** Every input is embedded in the body as a complete sub-file (frontmatter + body) wrapped in chiral order-aware delimiters. `decompose --full` recovers every leaf at byte precision, even after the original library files are gone.

## Steps that were run

1. **Scaffold** four atomic order-0 elements via `oovra create --new`:
   - `role-statement` — who the assistant is
   - `safety-fence` — refusal posture
   - `tone-discipline` — direct, no preamble
   - `format-rules` — markdown output rules
2. **Compose** them into one order-1 element `pairing-agent.md` (4 inputs).
3. **Move** the four originals out of the library into `originals-moved/`. Library now contains only `pairing-agent.md`.
4. **Decompose --full** the order-1 file, writing the result to `recovered/`.
5. **Diff** each recovered leaf against the stashed original.

## Folder layout

```
01-lossless-roundtrip/
├── library/
│   └── pairing-agent.md          ← the only file — all inputs embedded in its body
├── originals-moved/
│   ├── role-statement.md
│   ├── safety-fence.md
│   ├── tone-discipline.md
│   └── format-rules.md
└── recovered/
    └── pairing-agent/
        ├── pairing-agent.md      ← copy of the parent
        ├── role-statement.md
        ├── safety-fence.md
        ├── tone-discipline.md
        └── format-rules.md
```

## Why it works

The body of `pairing-agent.md` looks like:

```
~~>>
+++
(role-statement's full TOML header)
+++

(role-statement's body)
~~<<
~~>>
+++
(safety-fence's full header)
+++

(safety-fence's body)
~~<<
... (and so on for tone-discipline, format-rules)
```

`decompose` walks the body line-by-line, splits at `~~>>`/`~~<<` pairs, then runs the same `parse()` function on each chunk. Every chunk is itself a complete Oovra file — so the recovered leaf includes the original `name`, `version`, and `meta` fields verbatim. No external state needed.

## Verdict

See `results.md` — all 4 leaves byte-identical to the originals.

## See also

- [reference/command-decompose.md](../../../reference/command-decompose.md) — full `decompose` reference, including `--full` semantics
- [reference/command-compose.md](../../../reference/command-compose.md) — how the composed file is produced
- [reference/schema.md](../../../reference/schema.md) — the file format being round-tripped
- [reference/kind-and-delimiters.md](../../../reference/kind-and-delimiters.md) — the chiral delimiter scheme that makes byte-identity possible
