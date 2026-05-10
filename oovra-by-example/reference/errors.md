# Errors Reference

Oovra's errors are an **agent-facing API**. Each variant is structured (with explicit fields), each error message attaches the file path and explains the specific problem in actionable terms. The aim: an LLM reading the error can take a corrective action without further context.

This page lists every variant of `OovraError`, what triggers it, and the canonical error message you'll see.

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

The `<os error>` portion is whatever `std::io::Error` provided — typically "Permission denied", "No such file or directory", "Disk full", etc.

---

### `NotADirectory`

```rust
NotADirectory(PathBuf)
```

**Trigger**: `Library::load` was given a path that exists but is a file, not a directory.

**Example trigger**:

```bash
oovra compose --library ./elements/role-statement.md role-statement  # library should be a dir, not a file
```

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

**Example invalid file**:

```
+++
name = "Foo"
order = 0

body never starts because there's no closing +++
```

**Message**: `Missing closing '+++' delimiter in ./bad.md. Frontmatter must be terminated by '+++' on its own line.`

---

## Parsing — TOML

### `InvalidToml`

```rust
InvalidToml { path: PathBuf, source: toml::de::Error }
```

**Trigger**: the content between the two `+++` lines is not valid TOML.

**Example invalid file**:

```
+++
name = "Foo
order = 0
+++

body
```

(Note the unterminated string in `name`.)

**Message**: `Invalid TOML in frontmatter of ./bad.md: <toml-parse-error-with-line-and-column>`

The `<toml-parse-error>` portion is from the `toml` crate and includes a caret-pointer to the offending column. Very actionable for LLM agents.

---

### `TomlSerialize`

```rust
TomlSerialize { id: String, source: toml::ser::Error }
```

**Trigger**: would only fire if a `PromptElementHeader` struct contained a value the TOML serializer couldn't represent. In practice this should never happen because the struct fields are all serializable types — this is a defensive variant.

**Message**: `Failed to serialize TOML for element 'foo': <toml-ser-error>`

---

## Validation — required and constrained fields

### `MissingField`

```rust
MissingField { path: PathBuf, field: &'static str }
```

**Trigger**: a structurally-required field is missing during deserialization. Usually caught by `InvalidToml` first (because serde's missing-field error fires before this variant is reachable in current code), but kept as a defensive variant.

**Message**: `Missing required field 'name' in ./bad.md`

---

### `InvalidField`

```rust
InvalidField { path: PathBuf, field: &'static str, value: String, reason: String }
```

**Trigger**: a field had the wrong shape — `id` not kebab-case, `version` not semver, `generated_at` not RFC 3339, `name` empty, etc. The most-thrown variant during validation.

**Example invalid files and their messages**:

| Invalid input                                  | Message                                                                                                                                       |
|-----------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------|
| `id = "NotKebab"`                              | `Field 'id' in ./bad.md has invalid value 'NotKebab': must be kebab-case (lowercase letters, digits, hyphens; no leading/trailing/double hyphens)` |
| `version = "v1"`                               | `Field 'version' in ./bad.md has invalid value 'v1': must be valid semver (e.g. "1.0.0")`                                                     |
| `generated_at = "2026-05-10"` (no time)        | `Field 'generated_at' in ./bad.md has invalid value '2026-05-10': must be RFC 3339 (e.g. "2026-05-09T14:23:15Z")`                            |
| `name = ""`                                    | `Field 'name' in ./bad.md has invalid value '': must be non-empty`                                                                            |
| atomic file with `generated_at` set            | `Field 'generated_at\|render_mode\|body_level' in ./bad.md has invalid value '<set>': these fields are only valid when composed_of is also present` |

---

### `EmptyBody`

```rust
EmptyBody(PathBuf)
```

**Trigger**: the body (everything after the closing `+++` plus one blank line) is empty or contains only whitespace.

**Example invalid file**:

```
+++
name = "Foo"
order = 0
id = "foo"
version = "1.0.0"
meta = ""
+++


```

(Just whitespace where the body should be.)

**Message**: `Empty body in ./bad.md. The body must be non-empty after stripping whitespace.`

---

## Validation — joint invariants

### `OrderRequiresField`

```rust
OrderRequiresField { id: String, order: u32, field: &'static str }
```

**Trigger**: a file has `composed_of` set but is missing one of the required companion fields (`generated_at`, `render_mode`, or `body_level`). Each missing field triggers a separate error in turn (only the first missing field is reported).

**Example invalid file** (order-1 file missing `body_level`):

```
+++
name = "Composed Without Body Level"
order = 1
id = "broken"
version = "1.0.0"
meta = ""
generated_at = "2026-05-10T18:00:00Z"
render_mode = "markdown-h2"
composed_of = [{ id = "x", version = "1.0.0" }]
+++

body
```

**Message**: `Order-1 element 'broken' is missing the 'body_level' field, which is required for order >= 1`

(The error message name still references "order >= 1" because that's the most common case; the rule is actually "composed_of present implies all four companions present".)

---

### `HandAuthoredHigherOrder`

```rust
HandAuthoredHigherOrder { path: PathBuf, order: u32 }
```

**Trigger**: a file claims `order >= 1` but lacks a `composed_of` recipe. This is what prevents users from hand-authoring files that claim a compositional depth they don't actually have.

**Example invalid file**:

```
+++
name = "Fake Order 2"
order = 2
id = "fake"
version = "1.0.0"
meta = ""
+++

body
```

**Message**: `Hand-authored elements must be order 0; found order 2 in ./bad.md. Use 'oovra compose' to produce higher-order elements.`

---

## Library and resolution

### `DuplicateId`

```rust
DuplicateId { id: String, first: PathBuf, second: PathBuf }
```

**Trigger**: `Library::load` found two files in the same library directory with the same `id` in their frontmatter.

**Example trigger**: two files `library/role-v1.md` and `library/role-v2.md` both have `id = "role-statement"`.

**Message**: `Duplicate ID 'role-statement' in library: 'library/role-v1.md' and 'library/role-v2.md'`

---

### `ElementNotFound`

```rust
ElementNotFound { id: String }
```

**Trigger**: a CLI command (compose, compose --text, --re-render) referenced an ID that doesn't exist in the resolved library.

**Example trigger**:

```bash
oovra compose --library ./elements --text does-not-exist
```

**Message**: `Element 'does-not-exist' not found in library`

---

### `VersionMismatch`

```rust
VersionMismatch { id: String, pin: String, actual: String }
```

**Trigger**: `compose --re-render` enforces version pins from the existing file's `composed_of`. If the library has a different version of any pinned input, this fires.

**Example trigger**: `pairing-agent.md` pins `role-statement = "1.0.0"`, but `library/role-statement.md` has been bumped to `1.1.0`.

```bash
oovra compose --library ./elements --re-render ./elements/pairing-agent.md
```

**Message**: `Version mismatch for 'role-statement': pin '1.0.0' does not match library version '1.1.0'`

---

## Operator dispatch

### `EmptyCompose`

```rust
EmptyCompose
```

**Trigger**: `compose` was called programmatically (via `ComposeRequest`) with an empty `inputs` vector. The CLI prevents this at the argument-parsing level (clap requires at least one positional unless `--re-render` is used), so this variant is mostly for library consumers.

**Message**: `Compose requires at least one input`

---

### `AlreadyLabeled`

```rust
AlreadyLabeled(PathBuf)
```

**Trigger**: `create --label <PATH>` was given a path that already starts with `+++` (already has an Oovra header), and `--force` was not passed.

**Example trigger**:

```bash
# First time: succeeds, prepends header
oovra create --label ./doc.md --id my-doc --name "My Doc"

# Second time: refused
oovra create --label ./doc.md --id my-doc-v2 --name "My Doc v2"
# → Error: File ./doc.md already has an Oovra header. Use --force to overwrite.
```

**Message**: `File ./doc.md already has an Oovra header. Use --force to overwrite.`

---

### `CannotDecomposeAtomic`

```rust
CannotDecomposeAtomic { id: String }
```

**Trigger**: `decompose` (either mode) was called on a file without a `composed_of` recipe.

**Example trigger**:

```bash
oovra decompose ./elements/role-statement.md  # an atomic file
```

**Message**: `Cannot decompose atomic element 'role-statement'. Atomic elements have no recipe (no \`composed_of\` field). Only Compose-produced elements can be decomposed.`

---

### `BodyParse`

```rust
BodyParse { id: String, order: u32, reason: String }
```

**Trigger**: `decompose` tried to split a composed file's body at the level-`body_level` delimiters and encountered a structural problem — unmatched open, unmatched close, or no delimiters at all. Usually indicates the body has been hand-edited (against the rules in [command-compose.md](./command-compose.md#mode-1-file-default--write-a-composed-md)) or is corrupt.

**Possible `reason` values**:

- `"encountered '~~>>' while still inside an open chunk"` — two opens in a row
- `"encountered '~~<<' without a matching '~~>>'"` — close without prior open
- `"missing '~~<<' to close the final chunk"` — last chunk is unterminated
- `"no '~~>>' delimiters found in body"` — file claims to be composed but body has no chunks

**Message**: `Body of order-1 element 'foo' could not be split into the expected sub-element chunks: <reason>`

---

## Compare dispatch refusals

### `OrderMismatch`

```rust
OrderMismatch { a_id: String, a_order: u32, b_id: String, b_order: u32 }
```

**Trigger**: `compare A B` where `A.order != B.order`.

**Example trigger**: comparing an atomic order-0 file with an order-1 composed file.

**Message**: `Cannot compare elements of different orders: 'role-statement' is order 0, 'coding-agent' is order 1. Compare requires same-order inputs.`

---

### `AtomicityMismatch`

```rust
AtomicityMismatch { a_id: String, a_kind: &'static str, b_id: String, b_kind: &'static str }
```

**Trigger**: `compare A B` where both are at the same order, but one is atomic and the other is composed. This is specifically the `compose([single-input])` edge case — a composed file can be `order = 0` if its sole input was order-0.

**Example trigger**: comparing a hand-authored atomic against a `compose([atomic])` output of the same logical order.

**Message**: `Cannot compare an atomic element with a composed element: 'role-statement' is atomic, 'role-statement-wrapped' is composed. Compare requires both inputs to be the same kind (both atomic or both composed).`

---

## Quick lookup: which command produces which error?

| Error | `create` | `compose` | `decompose` | `compare` |
|---|---|---|---|---|
| `FileNotFound` | `--label` |  | ✓ | ✓ |
| `Io` / `WriteIo` | ✓ | ✓ | ✓ |  |
| `EmptyFile` / `MissingOpenDelimiter` / `MissingCloseDelimiter` |  | ✓ (library load) | ✓ | ✓ |
| `InvalidToml` |  | ✓ (library load) | ✓ | ✓ |
| `InvalidField` | ✓ | ✓ (library load) | ✓ | ✓ |
| `EmptyBody` | ✓ (post-write) | ✓ | ✓ | ✓ |
| `HandAuthoredHigherOrder` |  | ✓ (library load) | ✓ | ✓ |
| `OrderRequiresField` |  | ✓ (library load) | ✓ | ✓ |
| `DuplicateId` |  | ✓ |  |  |
| `ElementNotFound` |  | ✓ |  |  |
| `VersionMismatch` |  | `--re-render` |  |  |
| `EmptyCompose` |  | ✓ (library only) |  |  |
| `AlreadyLabeled` | `--label` |  |  |  |
| `CannotDecomposeAtomic` |  |  | ✓ |  |
| `BodyParse` |  |  | ✓ |  |
| `OrderMismatch` |  |  |  | ✓ |
| `AtomicityMismatch` |  |  |  | ✓ |
| `NotADirectory` |  | ✓ | ✓ |  |

---

## Why error messages are designed this way

Three principles:

1. **Always attach the file path.** A "TOML parse failed" message with no path is useless; "Missing closing `+++` delimiter in `./elements/foo.md`" is actionable.
2. **Name the specific field.** Don't say "validation failed"; say "Field 'version' in foo.md has invalid value 'v1': must be valid semver".
3. **Suggest the fix when obvious.** "Use `--force` to overwrite" / "Use `oovra compose` to produce higher-order elements" / "Compare requires same-order inputs". The error doesn't just say *no* — it tells you what the correct action was.

These three are particularly important when an LLM agent is the consumer. An agent that can read a clear error message can self-correct in the next turn; an agent that gets "TOML deserialization failed" has nothing to act on.

---

## See also

- [schema.md](./schema.md) — the rules these errors enforce
- [command-create.md](./command-create.md), [command-compose.md](./command-compose.md), [command-decompose.md](./command-decompose.md), [command-compare.md](./command-compare.md) — per-command failure-mode tables
