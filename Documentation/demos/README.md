# Demos

Five end-to-end demonstrations of distinct architectural properties of Oovra. Each subfolder is self-contained — open the `notes.md` for an explanation of what's being demonstrated and the `results.md` for the captured verdict.

Demos are use-case-driven: each one asks a question like "does the lossless decomposition claim actually hold byte-for-byte?" and answers it with concrete files and captured outputs. They complement the per-feature [reference docs](../reference/) — the reference tells you *how* each command works; the demos show you *that* the architecture works.

## The five demos

| # | Folder                                       | Demonstrates                                                                                                                          | Key file to read first |
|---|----------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| 1 | [`01-lossless-roundtrip/`](./01-lossless-roundtrip/) | A composed file is its own self-extracting archive — decompose recovers every leaf byte-for-byte after the source library is moved away | [`results.md`](./01-lossless-roundtrip/results.md) (4 SHA256 matches) |
| 2 | [`02-text-prose-output/`](./02-text-prose-output/)   | The same on-disk file has two external presentations: self-describing (with TOML + delimiters) vs prose (clean H2 sections)            | [`prose-form-output.md`](./02-text-prose-output/prose-form-output.md) |
| 3 | [`03-deep-text-flattening/`](./03-deep-text-flattening/) | `--text` recurses through every level; a `body_level = 2` compound flattens to the same shape as a `body_level = 1` compound            | [`prose-output.md`](./03-deep-text-flattening/prose-output.md) |
| 4 | [`04-structural-diff/`](./04-structural-diff/)       | `oovra compare` cuts through 26 lines of surface byte-level noise to identify two semantic version changes (and now reports `moved` inputs in v0.2) | [`compare-output-human.md`](./04-structural-diff/compare-output-human.md) |
| 5 | [`05-mixed-order-regression/`](./05-mixed-order-regression/) | The body_level escalation fix: composing a compound with atoms requires the outer delimiter level to climb above any inner element's   | [`delimiter-analysis.md`](./05-mixed-order-regression/delimiter-analysis.md) and [`results.md`](./05-mixed-order-regression/results.md) |

## Anatomy of each demo

Every demo follows the same shape:

```
NN-demo-name/
├── notes.md         — explanation: what's being demonstrated, why it matters, how it works under the hood
├── results.md       — captured outputs and verdicts: byte-equality assertions, line counts, file sizes, structural-diff results
├── library/         — the input/staged files the demo operates on (where applicable)
└── recovered/       — output files produced during the demo (where applicable, e.g. `decompose --full` outputs)
```

`notes.md` is the human-facing explanation. `results.md` is the machine-captured verdict — actual SHA256 sums, actual line counts, actual diff outputs. Together they tell you what was demonstrated and the evidence that it worked.

## Mapping each demo to an architectural decision

| Demo | Architectural decision being verified                                                                                                                |
|------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1    | **Body as source of truth, not header.** The composed body embeds full sub-element files. Decompose is lossless without library access.                |
| 2    | **Two renderers, one source.** `render_body` (delimited, lossless) and `render_text` (flattened, paste-ready) serve different audiences from the same data. |
| 3    | **Recursive flatten doesn't depend on depth.** `render_for_paste` is self-similar — split one level and recurse. Deep compounds flatten the same way as shallow ones. |
| 4    | **Recipes are first-class, not derived from prose.** `composed_of` is a structural index that survives every prose change, every timestamp shift, every body edit. v0.2's sequence-aware diff also catches reorderings. |
| 5    | **`body_level` must always strictly escalate.** The outer compound's delimiter tilde count must exceed every inner element's, regardless of whether the inputs are atoms or compounds. |

## Snapshot status

These demos were captured against **Oovra v0.1.0** and migrated in place to **v0.2** as part of the schema migration. The library files now carry `kind = "atom"` / `kind = "compound"` in their frontmatters; the rendered outputs and captured `results.md` content are unchanged because the migration is a representation change, not a regeneration.

If you re-run any demo today, you'll get fresh timestamps but identical structural outputs.

## See also

- [../reference/](../reference/) — feature-by-feature reference docs (what each command does, what each error means)
- [../README.md](../README.md) — the top-level Documentation navigation
- [../reference/build-guide.md](../reference/build-guide.md) — the from-first-principles construction guide
- [../../CHANGELOG.md](../../CHANGELOG.md) — v0.1 → v0.2 migration notes
