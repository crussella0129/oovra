# `oovra create` — Reference

Creates a new **atom** (`kind = "atom"`) prompt element, either from scratch (`--new`) or by labeling an existing Markdown file (`--label`).

> **v0.2 migration note.** This page documents the v0.2 schema. The `order = 0` integer is gone; atoms are now declared with `kind = "atom"`. If you have v0.1 files on disk, run `oovra migrate <library-dir>` to convert them in place.

## Synopsis

```
oovra create --new <ID>      [--name <NAME>] [--version <VERSION>] [--meta <META>] [--library <DIR>] [--id <ID>]
oovra create --label <PATH>  [--name <NAME>] [--version <VERSION>] [--meta <META>] [--id <ID>] [--force]
```

Exactly one of `--new` or `--label` must be passed.

## What it does

Both modes produce an **atom** (`kind = "atom"`, no `composed_of`). Atoms cannot be promoted to compounds by hand — that's [`oovra compose`](./command-compose.md)'s job. The parser actively rejects any hand-authored file claiming `kind = "compound"` without a real recipe.

The command validates its work in two ways:

1. **In-memory pre-write check**: before any disk write, the generated content is parsed through the full validator. If validation fails, *nothing is written* — no orphan file is ever created on disk.
2. **Post-write re-parse**: after the write succeeds, the file is read back and re-parsed. Catches filesystem-layer issues (encoding, line-endings) on the rare occasion they occur.

If either check fails, you get a [structured error](./errors.md) pointing to the specific problem.

---

## Mode 1: `--new <ID>` — scaffold from scratch

Creates a fresh `.md` file at `<library>/<id>.md` with the required frontmatter and a TODO-placeholder body, so the file is immediately a valid Oovra atom.

### Flags

| Flag        | Required?           | Default                                 | Purpose                                                                                                  |
|-------------|---------------------|-----------------------------------------|----------------------------------------------------------------------------------------------------------|
| `--new <ID>` | yes (in this mode)  | —                                       | The ID for the new element. Must be kebab-case. Also used as the filename (`<library>/<id>.md`).         |
| `--id <ID>` | no                  | the value of `--new`                    | Override the ID written into the frontmatter (rarely needed; allows decoupling filename from frontmatter ID, though they're usually the same). |
| `--name <NAME>` | no              | the value of `--new`                    | The human-readable name written into the `name` field. Defaults to the ID if not given.                  |
| `--version <V>` | no              | `"1.0.0"`                               | Semver string for the `version` field.                                                                   |
| `--meta <META>` | no              | `""`                                    | Free-form description for the `meta` field. May be empty.                                                |
| `--library <DIR>` | no            | `./elements`                            | Directory to write the new file into. The directory is created if it does not exist.                     |
| `--force`   | no                  | (off)                                   | Has no effect in `--new` mode; the command never overwrites in this mode regardless. (See `--label`.)    |

### What's produced

A file at `<library>/<id>.md` with this exact shape (modulo your flag values):

```toml
+++
name = "<NAME or ID>"
kind = "atom"
id = "<ID>"
version = "<VERSION>"
meta = "<META>"
+++

<!-- TODO: write the prompt body for `<ID>` here. This element is an atom — internally consistent, portable across compositions. -->
```

The TODO comment is the body. It's a valid Markdown comment, and the resulting file passes validation immediately — but you should replace it with actual prompt content before composing.

### Example

```bash
oovra create --new refusal-policy --library ./elements --name "Refusal Policy" --meta "Brief decline of harmful requests"
```

Result on disk at `./elements/refusal-policy.md`:

```toml
+++
name = "Refusal Policy"
kind = "atom"
id = "refusal-policy"
version = "1.0.0"
meta = "Brief decline of harmful requests"
+++

<!-- TODO: write the prompt body for `refusal-policy` here. This element is an atom — internally consistent, portable across compositions. -->
```

You then open the file in your editor and replace the TODO with the actual policy text.

### Failure modes

| Trigger                                       | Error                                                                                       |
|-----------------------------------------------|---------------------------------------------------------------------------------------------|
| `--new` value is not kebab-case               | [`InvalidField { field: "id", ... }`](./errors.md#invalidfield) — file is *not* written     |
| `--version` value is not semver               | [`InvalidField { field: "version", ... }`](./errors.md#invalidfield)                        |
| `<library>` exists but is not writable        | [`WriteIo { ... }`](./errors.md#io--writeio)                                                |
| `--new` not passed AND `--label` not passed   | clap argument error: "exactly one of --new or --label required"                             |

---

## Mode 2: `--label <PATH>` — promote an existing Markdown file

Takes an existing `.md` file and prepends an Oovra header in place. The file's existing prose becomes the body of an atom.

### Flags

| Flag             | Required?          | Default                         | Purpose                                                                                                          |
|------------------|--------------------|---------------------------------|------------------------------------------------------------------------------------------------------------------|
| `--label <PATH>` | yes (in this mode) | —                               | Path to an existing Markdown file (with or without an existing frontmatter block).                               |
| `--id <ID>`      | no                 | derived from `<PATH>`'s file stem | The ID for the labeled element. If omitted, uses the filename without extension. Must be kebab-case.            |
| `--name <NAME>`  | no                 | the ID                          | Human-readable name written into the frontmatter.                                                                |
| `--version <V>`  | no                 | `"1.0.0"`                       | Semver string.                                                                                                   |
| `--meta <META>`  | no                 | `""`                            | Free-form description.                                                                                           |
| `--library <DIR>`| no (ignored in --label) | —                          | The library-dir flag is accepted for symmetry but is not used by `--label`; the file is rewritten in place at its existing path. |
| `--force`        | no                 | (off)                           | Required if `<PATH>` already has an Oovra `+++` header — without `--force`, the command refuses to overwrite.    |

### What's produced

The file at `<PATH>` is rewritten in place. The new content is:

```toml
+++
name = "<NAME or ID>"
kind = "atom"
id = "<ID>"
version = "<VERSION>"
meta = "<META>"
+++

<the file's existing prose, preserved verbatim>
```

If the file was already an Oovra file and `--force` was passed, the *original* header is stripped and a new one is generated. The original body is preserved.

If the file was empty (rare), a TODO-placeholder body is inserted just like `--new` mode.

### `--force` semantics — what gets preserved and what doesn't

| File state before `--label`                  | Without `--force`                                          | With `--force`                                                                |
|----------------------------------------------|-----------------------------------------------------------|-------------------------------------------------------------------------------|
| No Oovra header (plain `.md`)                | Succeeds — prepends new header                            | Succeeds — same result                                                        |
| Already has Oovra `+++` header               | Refuses with [`AlreadyLabeled`](./errors.md#alreadylabeled) | Succeeds — strips old header, prepends new one, preserves body                |
| Empty file                                   | Succeeds — inserts TODO body                              | Succeeds — same result                                                        |

### Example

Starting file `./docs/refactor-rules.md`:

```markdown
Always finish a refactor by running the tests. If a test fails after a
refactor, the refactor is incomplete; revert or fix forward, never ship
a half-finished change.
```

Command:

```bash
oovra create --label ./docs/refactor-rules.md --name "Refactor Discipline" --meta "Refactors must end green" --version 1.0.0
```

Result at `./docs/refactor-rules.md`:

```toml
+++
name = "Refactor Discipline"
kind = "atom"
id = "refactor-rules"
version = "1.0.0"
meta = "Refactors must end green"
+++

Always finish a refactor by running the tests. If a test fails after a
refactor, the refactor is incomplete; revert or fix forward, never ship
a half-finished change.
```

ID was derived from the filename `refactor-rules.md` → `refactor-rules` (which is already kebab-case). If you wanted a different ID, pass `--id`.

### Failure modes

| Trigger                                                          | Error                                                                                |
|------------------------------------------------------------------|--------------------------------------------------------------------------------------|
| `<PATH>` does not exist                                          | [`FileNotFound`](./errors.md#filenotfound)                                           |
| Filename stem is not kebab-case AND `--id` not passed            | clap error or [`InvalidField`](./errors.md#invalidfield) (depending on which check fires first) |
| `<PATH>` already has an Oovra header AND `--force` not passed    | [`AlreadyLabeled`](./errors.md#alreadylabeled)                                       |
| Same validation failures as `--new` (bad ID, bad semver, etc.)   | corresponding [`InvalidField`](./errors.md#invalidfield) — file is **not** overwritten when validation fails |

---

## How `create` interacts with the library

After running `create`, the new file lives at `<library>/<id>.md` (or at its original path for `--label`). Both [`oovra compose`](./command-compose.md) and [`oovra decompose`](./command-decompose.md) accept a `--library <DIR>` flag pointing at the directory containing your `.md` files — they walk it recursively, so subdirectories are fine.

If two files in the same library have the same `id`, [`Library::load`](./errors.md#duplicateid) refuses to load it. The duplicate-detection is by frontmatter `id`, not filename — so if you `create --label` two different files with `--id same-thing`, the second file's load will fail.

---

## Why `create` always produces atoms

The parser's joint validator rejects any file with `kind = "compound"` that lacks a `composed_of` recipe. So if `create` produced compound files, they would be malformed by construction — they would lack a real recipe.

`compose` is the only path to a compound file because it's the only place where a real recipe (with real input versions) exists. This is intentional: it prevents authoring inconsistencies where someone hand-types a fake `composed_of` array referring to nodes that don't exist or are at different versions than claimed.

---

## See also

- [schema.md](./schema.md) — the file format `create` produces
- [command-compose.md](./command-compose.md) — how to combine atoms into compounds
- [errors.md](./errors.md) — all validation errors `create` can surface
- [demos/01-lossless-roundtrip](../demos/01-lossless-roundtrip/) — uses `create --new` to scaffold four atoms before composing them
