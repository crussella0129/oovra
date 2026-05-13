# Demo 2 — Prose Form via `compose --text`

## What this demonstrates

The same on-disk file has **two external presentations**, selected by which CLI flag you pass:
- `oovra decompose ...` / file-on-disk view → the **self-describing form** with TOML headers, `composed_of` recipe, and `~~>>`/`~~<<` delimiters wrapping every sub-element
- `oovra compose --text ...` → the **prose form** with all metadata stripped, just `## <id>` H2 sections and the bodies — the form you'd actually paste into Claude/GPT/etc.

These are not different files. They are the same file viewed through different renderers. The on-disk form is what makes round-trip semantics work; the prose form is what makes the tool useful as a daily-driver prompt composer.

## Steps that were run

1. Took `pairing-agent.md` (the order-1 element from Demo 1) — same file, no modifications.
2. Captured the first 30 lines of the raw form into `raw-form-excerpt.md` to show what the on-disk shape looks like.
3. Ran `oovra compose --library library --text pairing-agent` and captured the output into `prose-form-output.md`.

## Folder layout

```
02-text-prose-output/
├── library/
│   └── pairing-agent.md          ← the order-1 file, resolved by ID
├── pairing-agent.md              ← copy at the root, for direct inspection
├── raw-form-excerpt.md           ← first 30 lines of the on-disk file
└── prose-form-output.md          ← the --text rendering
```

## Side-by-side comparison

| Aspect | Raw form (on-disk) | Prose form (`--text`) |
|---|---|---|
| Outer TOML header | Present (`name`, `order`, `id`, `version`, `meta`, `generated_at`, `render_mode`, `body_level`, `composed_of`) | Stripped |
| Per-input TOML headers | Present (one for each embedded sub-element) | Stripped |
| Body delimiters | `~~>>`/`~~<<` lines | None |
| Section labels | None (each sub-element is delimited but not labeled inline) | `## <id>` Markdown H2 headers |
| Size | 1,709 bytes | ~830 bytes |
| Audience | The Rust tool, machines | LLMs, humans |

## What flips between them

The exact same prose bodies appear in both forms. What differs is the wrapping:
- **Raw**: every input's body sits inside its own `~~>>`/`~~<<` envelope, prefixed by that input's TOML frontmatter.
- **Prose**: every input's body sits under a `## <id>` Markdown H2 header. Frontmatter and delimiters are gone.

The prose form is **lossy by design** — version numbers, metadata, generation timestamps don't appear because they would be noise to a model. If you need the metadata back, decompose the original file.

## Implementation note

The prose form is produced by `render_text()` in `src/render.rs`, which calls `render_for_paste()` recursively. For atomic elements it emits `## <id>\n\n<body>`. For composed elements it splits the body into sub-elements (via `decompose()`) and recurses on each — so a deeply nested file flattens to a flat list of leaf sections regardless of how many levels of composition went into it. (See Demo 3 for an order-2 example of this recursion.)

## See also

- [reference/command-compose.md § Mode 2: --text](../../../reference/command-compose.md#mode-2---text--print-clean-prose-to-stdout) — full reference for the prose-output mode
- [reference/schema.md](../../../reference/schema.md) — what fields are present in the on-disk form vs stripped from the prose form
- [demos/03-deep-text-flattening](../03-deep-text-flattening/) — the same mechanism applied to a 2-level-deep composition
