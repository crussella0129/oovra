# Demos — v0.1 (migrated in place)

Five end-to-end demonstrations originally captured against **Oovra v0.1.0**. The library files in each demo have been migrated in place to v0.2 schema (the on-disk frontmatters now carry `kind` instead of `order`), but the rendered outputs and recorded `results.md` content are unchanged — migration is a representation change, not a regeneration.

Each subfolder is self-contained — open the `notes.md` for an explanation of what's being demonstrated and the `results.md` for the captured verdict.

## The five demos

| # | Folder                                       | Demonstrates                                                                                                                          | Key file to read first |
|---|----------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| 1 | [`01-lossless-roundtrip/`](./01-lossless-roundtrip/) | A compound is its own self-extracting archive — decompose recovers every leaf byte-for-byte after the source library is moved away | [`results.md`](./01-lossless-roundtrip/results.md) (4 SHA256 matches) |
| 2 | [`02-text-prose-output/`](./02-text-prose-output/)   | The same on-disk file has two external presentations: self-describing (with TOML + delimiters) vs prose (clean H2 sections)            | [`prose-form-output.md`](./02-text-prose-output/prose-form-output.md) |
| 3 | [`03-deep-text-flattening/`](./03-deep-text-flattening/) | `--text` recurses through every level; a `body_level = 2` compound flattens to the same shape as a `body_level = 1` compound            | [`prose-output.md`](./03-deep-text-flattening/prose-output.md) |
| 4 | [`04-structural-diff/`](./04-structural-diff/)       | `oovra compare` cuts through 26 lines of surface byte-level noise to identify two semantic version changes; v0.2's sequence-aware diff adds the `moved` axis on top | [`compare-output-human.md`](./04-structural-diff/compare-output-human.md) |
| 5 | [`05-mixed-order-regression/`](./05-mixed-order-regression/) | The body_level escalation rule: composing a compound with atoms requires the outer delimiter level to climb above any inner element's | [`delimiter-analysis.md`](./05-mixed-order-regression/delimiter-analysis.md) and [`results.md`](./05-mixed-order-regression/results.md) |

## Why the demos still work after migration

These demos were captured before v0.2 existed. After `oovra migrate` rewrote every Oovra file in this tree:

- The on-disk frontmatters now have `kind = "atom"` / `kind = "compound"` instead of `order = N`.
- The body content (and the chiral delimiter scheme) is unchanged.
- `compose --text`, `decompose --full`, and `compare` all produce the same outputs (modulo `generated_at` timestamps) they did under v0.1.

The narrative descriptions in each `notes.md` use v0.1-era vocabulary in places — they describe the architectural decisions as they were originally framed. The v0.2 vocabulary (atom / compound / kind / depth) maps cleanly onto the v0.1 framing; see [../../reference/schema.md](../../reference/schema.md) for the translation.

## See also

- [../v0.2/](../v0.2/) — v0.2-era demos (operator-gamut tests and v0.2-specific feature demonstrations)
- [../README.md](../README.md) — top-level demos index
- [../../reference/](../../reference/) — current reference docs
