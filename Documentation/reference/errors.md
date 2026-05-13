# Errors Reference

Oovra's errors are an **agent-facing API**. Each variant is structured (with explicit fields), each error message attaches the file path and explains the specific problem in actionable terms. The aim: an LLM reading the error can take a corrective action without further context.

This page lists every variant of `OovraError`, what triggers it, and the canonical error message you'll see in v0.2.

> **v0.2 migration note** — five v0.1 variants were renamed or removed in v0.2: `OrderRequiresField` → `MissingField` / `CompoundMissingField`; `HandAuthoredHigherOrder` removed (the case is now caught by `InvalidToml`/`AtomHasForbiddenField`); `OrderMismatch` removed (no `order` to mismatch); `AtomicityMismatch` → `KindMismatch`; `CannotDecomposeAtomic` → `CannotDecomposeAtom`. New variants `AtomHasForbiddenField` and `CompoundMissingField` give kind-aware messages.

## How errors surface in the CLI

The CLI uses `anyhow` at the application boundary, which wraps `OovraError` with command context. A typical CLI failure looks like:

```
Error: reading ./elements/foo.md

Caused by:
    Missing opening '+++' delimiter on line 1 of ./elements/foo.md. Expected '+++', got '... actual line ...'.
```

The top line is the high-level operation that failed; the `Caused by` chain shows the underlying `OovraError`. For agents parsing errors programmatically, the `Caused by` line is the structured one.

---

## Filesystem and I/O

### `FileNotFound`

```rust
FileNotFound(PathBuf)
```

**Trigger**: a CLI command was given a `<PATH>` argument pointing to a file that does not exist, or `Library::load` was given a non-existent directory.

**Example trigger**:

```bash
oovra decompose ./does-not-exist.md
```

**Message**: `File not found: ./does-not-exist.md`

---

### `Io` / `WriteIo`

```rust
Io { path: PathBuf, source: io::Error }
WriteIo { path: PathBuf, source: io::Error }
```

**Trigger**: any read/write failure from the underlying OS — permission denied, disk full, file locked by another process, etc. `Io` is for reads; `WriteIo` is for writes (including `mkdir`).

**Example trigger**: trying to write into a read-only directory.

**Message**: `Failed to read /some/path: <os error>` or `Failed to write /some/path: <os error>`

---

### `NotADirectory`

```rust
NotADirectory(PathBuf)
```

**Trigger**: `Library::load` or `migrate_library` was given a path that exists but is a file, not a directory.

**Message**: `'./elements/role-statement.md' is not a directory`

---

## Parsing — frontmatter delimiters

### `EmptyFile`

```rust
EmptyFile(PathBuf)
```

**Trigger**: file exists but is zero bytes (or trim-empty).

**Message**: `Empty file: ./empty.md`

---

### `MissingOpenDelimiter`

```rust
MissingOpenDelimiter { path: PathBuf, actual: String }
```

**Trigger**: the first line of the file is not exactly `+++`. The `actual` field captures what was actually on line 1.

**Example invalid file** (`./bad.md`):

```
name = "Foo"
+++

body
```

**Message**: `Missing opening '+++' delimiter on line 1 of ./bad.md. Expected '+++', got 'name = "Foo"'.`

---

### `MissingCloseDelimiter`

```rust
MissingCloseDelimiter(PathBuf)
```

**Trigger**: the parser found an opening `+++` but never found a closing `+++` on its own line.

**Message**: `Missing closing '+++' delimiter in ./bad.md. Frontmatter must be terminated by '+++' on its own line.`

---

## Parsing — TOML

### `InvalidToml`

```rust
InvalidToml { path: PathBuf, source: toml::de::Error }
```

**Trigger**: the content between the two `+++` lines is not valid TOML *or* a required structural field (including `kind`) is missing *or* `kind` has an invalid value.

**Example v0.2-specific triggers**:

| Invalid frontmatter                                          | Underlying serde message                                              |
|-------------------------------------------------------------|-----------------------------------------------------------------------|
| `name = "Foo"\nid = "foo"\nversion = "1.0.0"\nmeta = ""`    | `missing field 'kind'`                                                |
| `kind = "atomic"` (invalid value)                            | `unknown variant 'atomic', expected 'atom' or 'compound'`             |
| `name = "Foo`  (unterminated string)                         | TOML parse error with caret-pointer to the offending column           |

**Message**: `Invalid TOML in frontmatter of ./bad.md: <toml-parse-error-with-line-and-column>`

---

### `TomlSerialize`

```rust
TomlSerialize { id: String, source: toml::ser::Error }
```

**Trigger**: defensive variant; would only fire if a `PromptElementHeader` struct contained a value the TOML serializer couldn't represent.

**Message**: `Failed to serialize TOML for element 'foo': <toml-ser-error>`

---

## Validation — required and constrained fields

### `MissingField`

```rust
MissingField { path: PathBuf, field: &'static str }
```

**Trigger**: a structurally-required field is missing during deserialization but slipped past TOML parsing. Mostly defensive in v0.2 — the upstream `InvalidToml` usually catches missing fields first.

**Message**: `Missing required field 'name' in ./bad.md`

---

### `InvalidField`

```rust
InvalidField { path: PathBuf, field: &'static str, value: String, reason: String }
```

**Trigger**: a field had the wrong shape — `id` not kebab-case, `version` not semver, `generated_at` not RFC 3339, `name` empty, `composed_of` empty array, etc.

**Example invalid files and their messages**:

| Invalid input                                  | Message                                                                                                                                       |
|-----------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|
| `id = "NotKebab"`                              | `Field 'id' in ./bad.md has invalid value 'NotKebab': must be kebab-case (lowercase letters, digits, hyphens; no leading/trailing/double hyphens)` |
| `version = "v1"`                               | `Field 'version' in ./bad.md has invalid value 'v1': must be valid semver (e.g. "1.0.0")`                                                     |
| `generated_at = "2026-05-10"` (no time)        | `Field 'generated_at' in ./bad.md has invalid value '2026-05-10': must be RFC 3339 (e.g. "2026-05-09T14:23:15Z")`                            |
| `name = ""`                                    | `Field 'name' in ./bad.md has invalid value '': must be non-empty`                                                                            |
| `composed_of = []`                             | `Field 'composed_of' in ./bad.md has invalid value '[]': compounds must have at least one input`                                              |
| `body_level = 0` on a compound                 | `Field 'body_level' in ./bad.md has invalid value '0': must be >= 1 for compounds`                                                            |
| `depth = 0` on a compound                      | `Field 'depth' in ./bad.md has invalid value '0': must be >= 1 for compounds`                                                                 |

---

### `EmptyBody`

```rust
EmptyBody(PathBuf)
```

**Trigger**: the body (everything after the closing `+++` plus one blank line) is empty or contains only whitespace.

**Message**: `Empty body in ./bad.md. The body must be non-empty after stripping whitespace.`

---

## Validation — kind-specific invariants

### `AtomHasForbiddenField`

```rust
AtomHasForbiddenField { path: PathBuf, id: String, field: &'static str }
```

**Trigger**: a file with `kind = "atom"` carries a compound-only field. The forbidden fields are `composed_of`, `generated_at`, `render_mode`, `body_level`, and `depth`.

**Example invalid file**:

```toml
+++
name = "Bad Atom"
kind = "atom"
id = "bad"
version = "1.0.0"
meta = ""
body_level = 1
+++

body
```

**Message**: `Atom 'bad' in ./bad.md has forbidden field 'body_level'. Atoms have no recipe and no composition metadata.`

---

### `CompoundMissingField`

```rust
CompoundMissingField { path: PathBuf, id: String, field: &'static str }
```

**Trigger**: a file with `kind = "compound"` is missing one of the required companion fields (`composed_of`, `generated_at`, `render_mode`, `body_level`). The first missing field is reported; fix it and re-validate to see the next.

**Example invalid file** (compound missing `body_level`):

```toml
+++
name = "Bad Compound"
kind = "compound"
id = "broken"
version = "1.0.0"
meta = ""
generated_at = "2026-05-10T18:00:00Z"
render_mode = "markdown-h2"
composed_of = [{ id = "x", version = "1.0.0" }]
+++

body
```

**Message**: `Compound 'broken' in ./bad.md is missing required field 'body_level'.`

---

## Library and resolution

### `DuplicateId`

```rust
DuplicateId { id: String, first: PathBuf, second: PathBuf }
```

**Trigger**: `Library::load` found two files in the same library directory with the same `id` in their frontmatter.

**Message**: `Duplicate ID 'role-statement' in library: 'library/role-v1.md' and 'library/role-v2.md'`

---

### `ElementNotFound`

```rust
ElementNotFound { id: String }
```

**Trigger**: a CLI command (compose, compose --text, --re-render) referenced an ID that doesn't exist in the resolved library.

**Message**: `Element 'does-not-exist' not found in library`

---

### `VersionMismatch`

```rust
VersionMismatch { id: String, pin: String, actual: String }
```

**Trigger**: `compose --re-render` enforces version pins from the existing file's `composed_of`. If the library has a different version of any pinned input, this fires.

**Message**: `Version mismatch for 'role-statement': pin '1.0.0' does not match library version '1.1.0'`

---

## Operator dispatch

### `EmptyCompose`

```rust
EmptyCompose
```

**Trigger**: `compose` was called programmatically with an empty `inputs` vector. The CLI prevents this at argument-parsing.

**Message**: `Compose requires at least one input`

---

### `AlreadyLabeled`

```rust
AlreadyLabeled(PathBuf)
```

**Trigger**: `create --label <PATH>` was given a path that already starts with `+++` (already has an Oovra header), and `--force` was not passed.

**Message**: `File ./doc.md already has an Oovra header. Use --force to overwrite.`

---

### `CannotDecomposeAtom`

```rust
CannotDecomposeAtom { id: String }
```

**Trigger**: `decompose` (either mode) was called on an atom (a file with no `composed_of` recipe).

**Example trigger**:

```bash
oovra decompose ./elements/role-statement.md  # an atom
```

**Message**: `Cannot decompose atom 'role-statement'. Atoms have no recipe. Only compounds can be decomposed.`

---

### `BodyParse`

```rust
BodyParse { id: String, body_level: u32, reason: String }
```

**Trigger**: `decompose` tried to split a compound's body at the level-`body_level` delimiters and encountered a structural problem — unmatched open, unmatched close, or no delimiters at all. Usually indicates the body has been hand-edited (against the rules in [command-compose.md](./command-compose.md)) or is corrupt.

**Possible `reason` values**:

- `"encountered '~~>>' while still inside an open chunk"` — two opens in a row
- `"encountered '~~<<' without a matching '~~>>'"` — close without prior open
- `"missing '~~<<' to close the final chunk"` — last chunk is unterminated
- `"no '~~>>' delimiters found in body"` — file claims to be a compound but body has no chunks

**Message**: `Body of compound 'foo' at body_level 1 could not be split into the expected sub-element chunks: <reason>`

---

## Compare dispatch refusals

### `KindMismatch`

```rust
KindMismatch { a_id: String, a_kind: &'static str, b_id: String, b_kind: &'static str }
```

**Trigger**: `compare A B` where one is an atom and the other is a compound. v0.1's separate `OrderMismatch` and `AtomicityMismatch` variants collapsed into this single mismatch in v0.2 — kind is the only axis of structural disagreement that compare needs to refuse.

**Example trigger**: comparing an atom against a compound.

**Message**: `Cannot compare an atom with a compound: 'role-statement' is an atom, 'coding-agent' is a compound.`

---

## Quick lookup: which command produces which error?

| Error | `create` | `compose` | `decompose` | `compare` | `migrate` |
|---|---|---|---|---|---|
| `FileNotFound` | `--label` |  | ✓ | ✓ | ✓ |
| `Io` / `WriteIo` | ✓ | ✓ | ✓ |  | ✓ |
| `NotADirectory` |  | ✓ | ✓ |  | ✓ |
| `EmptyFile` / `MissingOpenDelimiter` / `MissingCloseDelimiter` |  | ✓ (library load) | ✓ | ✓ | ✓ |
| `InvalidToml` |  | ✓ (library load) | ✓ | ✓ | ✓ |
| `InvalidField` | ✓ | ✓ (library load) | ✓ | ✓ | ✓ |
| `AtomHasForbiddenField` | ✓ (post-write) | ✓ (library load) | ✓ | ✓ | ✓ |
| `CompoundMissingField` |  | ✓ (library load) | ✓ | ✓ | ✓ |
| `EmptyBody` | ✓ (post-write) | ✓ | ✓ | ✓ | ✓ |
| `DuplicateId` |  | ✓ |  |  |  |
| `ElementNotFound` |  | ✓ |  |  |  |
| `VersionMismatch` |  | `--re-render` |  |  |  |
| `EmptyCompose` |  | ✓ (library only) |  |  |  |
| `AlreadyLabeled` | `--label` |  |  |  |  |
| `CannotDecomposeAtom` |  |  | ✓ |  |  |
| `BodyParse` |  |  | ✓ |  | ✓ (when migrating compound bodies) |
| `KindMismatch` |  |  |  | ✓ |  |
| `MissingField` |  | ✓ (library load) | ✓ | ✓ | ✓ |

---

## Why error messages are designed this way

Three principles:

1. **Always attach the file path.** A "TOML parse failed" message with no path is useless; "Missing closing `+++` delimiter in `./elements/foo.md`" is actionable.
2. **Name the specific field.** Don't say "validation failed"; say "Field 'version' in foo.md has invalid value 'v1': must be valid semver".
3. **Suggest the fix when obvious.** "Use `--force` to overwrite" / "Compounds must have at least one input". The error doesn't just say *no* — it tells you what the correct action was.

These three are particularly important when an LLM agent is the consumer. An agent that can read a clear error message can self-correct in the next turn; an agent that gets "TOML deserialization failed" has nothing to act on.

---

## See also

- [schema.md](./schema.md) — the rules these errors enforce
- [command-create.md](./command-create.md), [command-compose.md](./command-compose.md), [command-decompose.md](./command-decompose.md), [command-compare.md](./command-compare.md) — per-command failure-mode tables
- [../../CHANGELOG.md](../../CHANGELOG.md) — v0.1 → v0.2 error-variant renames
