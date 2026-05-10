# Demos

Five end-to-end demonstrations of distinct architectural properties of Oovra. Each subfolder is self-contained — open the `notes.md` for an explanation of what's being demonstrated and the `results.md` for the captured verdict.

Demos are use-case-driven: each one asks a question like "does the lossless decomposition claim actually hold byte-for-byte?" and answers it with concrete files and captured outputs. They complement the per-feature [reference docs](../reference/) — the reference tells you *how* each command works; the demos show you *that* the architecture works.

## The five demos

| # | Folder                                       | Demonstrates                                                                                                                          | Key file to read first |
|---|----------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| 1 | [`01-lossless-roundtrip/`](./01-lossless-roundtrip/) | A composed file is its own self-extracting archive — decompose recovers every leaf byte-for-byte after the source library is moved away | [`results.md`](./01-lossless-roundtrip/results.md) (4 SHA256 matches) |
| 2 | [`02-text-prose-output/`](./02-text-prose-output/)   | The same on-disk file has two external presentations: self-describing (with TOML + delimiters) vs prose (clean H2 sections)            | [`prose-form-output.md`](./02-text-prose-output/prose-form-output.md) |
| 3 | [`03-deep-text-flattening/`](./03-deep-text-flattening/) | `--text` recurses through every level; an order-2 file flattens to the same shape as an order-1 file                                    | [`prose-output.md`](./03-deep-text-flattening/prose-output.md) |
| 4 | [`04-structural-diff/`](./04-structural-diff/)       | `oovra compare` cuts through 26 lines of surface byte-level noise to identify two semantic version changes                              | [`compare-output-human.md`](./04-structural-diff/compare-output-human.md) |
| 5 | [`05-mixed-order-regression/`](./05-mixed-order-regression/) | The body_level escalation fix: `compose([order-1, order-0, order-0])` produces `order=1` but `body_level=2` — the two integers diverge   | [`delimiter-analysis.md`](./05-mixed-order-regression/delimiter-analysis.md) and [`results.md`](./05-mixed-order-regression/results.md) |

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
| 3    | **Recursive flatten doesn't depend on depth.** `render_for_paste` is self-similar — split one level and recurse. Order-N elements flatten the same way as order-1. |
| 4    | **Recipes are first-class, not derived from prose.** `composed_of` is a structural index that survives every prose change, every timestamp shift, every body edit. |
| 5    | **Logical order ≠ physical delimiter level.** Decoupling the user's count-based formula from the parser's strict-escalation requirement preserves both. |

## Snapshot status

These demos are snapshots captured against **Oovra v0.1.0** at the time of commit. When the format or operators change in a future version, the demos will be re-run against the new code and the captured outputs (SHA256 sums, file sizes, etc.) updated.

This means a successful demo run is **dated evidence** — see the timestamps in each demo's composed file frontmatter (`generated_at`) for when each artifact was produced. If you re-run the demo today, you'll get different timestamps but identical structural outputs.

## See also

- [../reference/](../reference/) — feature-by-feature reference docs (what each command does, what each error means)
- [../README.md](../README.md) — the top-level oovra-by-example navigation
