# Reference

The complete feature reference for Oovra v0.1. Each document is a stand-alone reference for one aspect of the system, with explicit specifications, every flag, every constraint, every error variant, and worked examples.

If you're new to Oovra, start at the parent [oovra-by-example/README.md](../README.md) and walk through the [demos](../demos/) first — they give you a concrete tour. This reference is for when you need to look something up: "what does this flag do," "what error can fire here," "what's the formula for `body_level`."

## Reading order

For first-time readers wanting a top-to-bottom tour of the reference:

1. **[schema.md](./schema.md)** — the file format itself. The required and conditional fields, validation rules, joint invariants. Read this first; everything else assumes you know what a valid Oovra file looks like.

2. **[order-and-delimiters.md](./order-and-delimiters.md)** — the two-integer system (`order` and `body_level`) and the chiral delimiter scheme. Foundational for understanding how composition and decomposition actually work mechanically.

3. **[command-create.md](./command-create.md)** — how to author atomic elements (the only thing you can hand-author).

4. **[command-compose.md](./command-compose.md)** — how to combine atomics (and previously-composed elements) into higher-order compositions. Covers all three modes: file output, `--text` prose, and `--re-render`.

5. **[command-decompose.md](./command-decompose.md)** — how to recover inputs from a composed file. Covers the one-level inspection (default and `--format=json`) and the recursive `--full` tree extraction.

6. **[command-compare.md](./command-compare.md)** — how to diff two files. Covers the four-way dispatch (content, structural, order mismatch, atomicity mismatch).

7. **[errors.md](./errors.md)** — every `OovraError` variant catalogued with trigger conditions and canonical messages. Useful both as a debugging reference and for agents that need to parse error output programmatically.

## Lookup-by-task

| I want to...                                                            | Go to                                                    |
|-------------------------------------------------------------------------|----------------------------------------------------------|
| Understand what an Oovra file looks like                                | [schema.md](./schema.md)                                 |
| Know what fields are required when                                      | [schema.md § joint invariant](./schema.md#field-presence-invariants-the-joint-validator) |
| Compose elements together                                                | [command-compose.md](./command-compose.md)               |
| Get a paste-ready prompt for an LLM                                      | [command-compose.md § Mode 2: --text](./command-compose.md#mode-2---text--print-clean-prose-to-stdout) |
| Recover the inputs of a composed file                                    | [command-decompose.md](./command-decompose.md)           |
| Compare two prompts and find what changed                                | [command-compare.md](./command-compare.md)               |
| Look up what an error message means                                      | [errors.md](./errors.md)                                 |
| Understand the difference between `order` and `body_level`               | [order-and-delimiters.md](./order-and-delimiters.md)     |
| Add a new element to an existing library                                 | [command-create.md § --new mode](./command-create.md#mode-1---new-id--scaffold-from-scratch) |
| Convert an existing Markdown file to an Oovra element                    | [command-create.md § --label mode](./command-create.md#mode-2---label-path--promote-an-existing-markdown-file) |
| Get JSON output for use with `jq` or scripts                             | [command-decompose.md](./command-decompose.md) (`--format=json`) and [command-compare.md](./command-compare.md) (`--format=json`) |

## Versioning

This reference is for **Oovra v0.1**. The schema is committed to backward compatibility within the v0.1 line — new optional fields may be added, but existing field types and semantics will not change.

When v0.2 ships (with planned additions like Claude-XML render modes, semver range matching on version pins, the `bundle` kind, etc.), this reference will be updated as a snapshot for that version. Look at the git history of `oovra-by-example/reference/` to see how the docs have changed across versions.

## See also

- [../demos/](../demos/) — end-to-end demonstrations of each property in action
- [../../SCHEMA.md](../../SCHEMA.md) — the canonical agent-facing schema contract (terser than this reference; written for LLM consumption)
- [../../oovra-build-guide.md](../../oovra-build-guide.md) — the from-first-principles construction guide for builders reimplementing this from scratch
