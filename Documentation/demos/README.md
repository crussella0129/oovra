# Demos

End-to-end demonstrations of distinct architectural properties of Oovra, organized by the version they were originally captured under.

```
demos/
├── README.md            ← this file (the demos index)
├── v0.1/                ← five demos captured against Oovra v0.1.0; library files migrated in place to v0.2
└── v0.2/                ← v0.2-era demos that exercise the new schema and operators
```

Each subfolder is self-contained — open its `README.md` for the index and per-demo `notes.md` for the narrative explanation. `results.md` files record the captured outputs (SHA256 hashes, line counts, byte-equality verdicts) so the demos double as regression evidence.

## v0.1 demos — architectural properties

The five v0.1 demos verify the load-bearing architectural decisions made when Oovra was first built. They were captured before v0.2 existed and migrated in place by `oovra migrate`. The narrative descriptions still reference v0.1-era vocabulary in places; the on-disk files are valid v0.2.

See [`v0.1/README.md`](./v0.1/README.md) for the full per-demo table. Headline results:

| # | What's verified |
|---|----------------|
| 1 | Compounds are self-extracting archives — decompose works without library access |
| 2 | One file has two external presentations (`--text` vs raw) |
| 3 | Recursive flatten doesn't care about depth |
| 4 | Recipes are first-class — diff cuts through prose-level noise |
| 5 | `body_level` must always strictly escalate to prevent parser collisions |

## v0.2 demos — operator-gamut and new-feature tests

Demos written specifically for v0.2. See [`v0.2/README.md`](./v0.2/README.md):

| # | What's verified |
|---|----------------|
| 1 | All four operators end-to-end via CLI; compare correctly fires on `added`, `removed`, `version_changed`, and `moved` simultaneously, with the spec-promised non-mutual-exclusivity of `version_changed` and `moved` |

Future v0.2-era demos can be added under the same folder; the `NN-name/` naming convention preserves a stable ordering.

## Anatomy of a demo

Every demo follows the same shape:

```
NN-demo-name/
├── notes.md         — explanation: what's being demonstrated, why it matters
├── results.md       — captured outputs and verdicts: hashes, byte-equality checks, diff outputs
├── library/         — input fixtures (or `library-v1/`, `library-v2/` when the demo exercises a mutation)
└── outputs/         — recovered/produced artifacts (compose outputs, decompose trees, JSON dumps)
```

`notes.md` is the human-facing explanation. `results.md` is the machine-captured verdict.

## Snapshot status

- v0.1 demos: captured against Oovra v0.1.0; library files migrated in place to v0.2 schema. Original behavior preserved; if you re-run any demo today you get fresh timestamps but identical structural outputs.
- v0.2 demos: captured against Oovra v0.2.0 (commit `3b34b25` or later). Reproducible via the commands embedded in each demo's `results.md` § "Reproducibility".

## See also

- [../reference/](../reference/) — feature-by-feature reference docs (what each command does, what each error means)
- [../README.md](../README.md) — the top-level Documentation navigation
- [../reference/build-guide.md](../reference/build-guide.md) — the from-first-principles construction guide
- [../../CHANGELOG.md](../../CHANGELOG.md) — v0.1 → v0.2 migration notes
