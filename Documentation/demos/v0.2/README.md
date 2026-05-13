# Demos — v0.2

Tests and demonstrations captured against **Oovra v0.2.0**. Each subfolder is a self-contained scenario that runs real `oovra` CLI invocations against real on-disk fixtures and captures the outputs.

The v0.2-era demos focus on what v0.2 added vs v0.1: the `kind` discriminator, the `depth` field, the `oovra migrate` subcommand, and **sequence-aware compare** (the new `moved` axis in structural diff).

## The demos

| # | Folder | Demonstrates | Headline result |
|---|--------|--------------|-----------------|
| 1 | [`01-operator-gamut/`](./01-operator-gamut/) | All four operators end-to-end (create, compose, decompose, compare) with a worst-case compare scenario — two compounds that differ on every diff axis | Compare correctly reports `added`, `removed`, `version_changed`, and `moved` simultaneously; `refusal-policy` fires on both `version_changed` and `moved` to verify the axes are not mutually exclusive |

More v0.2-era demos can be added here as the tool acquires new capabilities. The naming convention `NN-name/` matches v0.1's pattern and preserves a stable ordering for future readers.

## Anatomy of a v0.2 demo

```
NN-demo-name/
├── notes.md         — explanation: what's being demonstrated, why it matters
├── results.md       — captured outputs with SHA256 hashes and byte-equality verdicts
├── library-v1/      — fixtures (or `library/` for single-library scenarios)
├── library-v2/      — second library when the demo exercises a library mutation
└── outputs/         — captured artifacts: composed files, decompose trees, compare JSON, etc.
```

`notes.md` is the human-facing explanation. `results.md` is the machine-captured verdict — actual SHA256 sums, actual CLI output, actual byte-identity checks. Together they tell you what was demonstrated and the evidence that it worked.

## See also

- [../v0.1/](../v0.1/) — v0.1-era demos (migrated in place to v0.2 schema but originally captured against v0.1 semantics)
- [../README.md](../README.md) — top-level demos index
- [../../reference/](../../reference/) — the v0.2 reference docs the demos verify
- [../../../CHANGELOG.md](../../../CHANGELOG.md) — what v0.2 changed
