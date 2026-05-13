# Reference

The complete feature reference for Oovra v0.2. Each document is a stand-alone reference for one aspect of the system, with explicit specifications, every flag, every constraint, every error variant, and worked examples.

If you're new to Oovra, start at the parent [Documentation/README.md](../README.md) and walk through the [demos](../demos/) first — they give you a concrete tour. This reference is for when you need to look something up: "what does this flag do," "what error can fire here," "what's the formula for `body_level`."

## Reading order

For first-time readers wanting a top-to-bottom tour of the reference:

1. **[schema.md](./schema.md)** — the file format itself. The required fields, the `kind = "atom" | "compound"` discriminator, validation rules, joint invariants. Read this first; everything else assumes you know what a valid Oovra file looks like.

2. **[kind-and-delimiters.md](./kind-and-delimiters.md)** — the `body_level` integer and the chiral delimiter scheme. Foundational for understanding how composition and decomposition actually work mechanically.

3. **[command-create.md](./command-create.md)** — how to author atoms (the only thing you can hand-author).

4. **[command-compose.md](./command-compose.md)** — how to combine atoms (and previously-composed compounds) into deeper compounds. Covers all three modes: file output, `--text` prose, and `--re-render`.

5. **[command-decompose.md](./command-decompose.md)** — how to recover inputs from a compound. Covers the one-level inspection (default and `--format=json`) and the recursive `--full` tree extraction.

6. **[command-compare.md](./command-compare.md)** — how to diff two files. Covers the kind-aware dispatch (content vs structural) and the new sequence-aware diff that surfaces `moved` inputs.

7. **[errors.md](./errors.md)** — every `OovraError` variant catalogued with trigger conditions and canonical messages. Useful both as a debugging reference and for agents that need to parse error output programmatically.

## Lookup-by-task

| I want to...                                                            | Go to                                                    |
|-------------------------------------------------------------------------|----------------------------------------------------------|
| Understand what an Oovra file looks like                                | [schema.md](./schema.md)                                 |
| Know what fields are required when                                      | [schema.md § joint invariant](./schema.md#field-presence-invariants-the-joint-validator) |
| Compose elements together                                                | [command-compose.md](./command-compose.md)               |
| Get a paste-ready prompt for an LLM                                      | [command-compose.md § Mode 2: --text](./command-compose.md#mode-2---text--print-clean-prose-to-stdout) |
| Recover the inputs of a compound                                         | [command-decompose.md](./command-decompose.md)           |
| Compare two prompts and find what changed (including reorder)            | [command-compare.md](./command-compare.md)               |
| Look up what an error message means                                      | [errors.md](./errors.md)                                 |
| Understand `body_level` and `depth`                                      | [kind-and-delimiters.md](./kind-and-delimiters.md)       |
| Add a new atom to an existing library                                    | [command-create.md § --new mode](./command-create.md#mode-1---new-id--scaffold-from-scratch) |
| Convert an existing Markdown file to an atom                             | [command-create.md § --label mode](./command-create.md#mode-2---label-path--promote-an-existing-markdown-file) |
| Migrate a v0.1 library to v0.2                                           | Run `oovra migrate <library-dir>`; see [schema.md § Migrating from v0.1](./schema.md#migrating-from-v01) |
| Get JSON output for use with `jq` or scripts                             | [command-decompose.md](./command-decompose.md) (`--format=json`) and [command-compare.md](./command-compare.md) (`--format=json`) |

## Versioning

This reference is for **Oovra v0.2**. The breaking change from v0.1 was the schema discriminator: numeric `order` was replaced with explicit `kind = "atom" | "compound"`. The [CHANGELOG](../../CHANGELOG.md) summarises the full set of changes; [v0.2 SPEC](../version-reports/v0.1/SPEC-v0.2.md) is the design document.

## See also

- [../demos/](../demos/) — end-to-end demonstrations of each property in action
- [build-guide.md](./build-guide.md) — the from-first-principles construction guide for builders reimplementing this from scratch
- [../../CHANGELOG.md](../../CHANGELOG.md) — version history
- [../version-reports/v0.1/SPEC-v0.2.md](../version-reports/v0.1/SPEC-v0.2.md) — the v0.2 migration specification
