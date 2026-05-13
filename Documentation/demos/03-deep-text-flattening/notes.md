# Demo 3 — Deep `--text` Recursive Flattening

## What this demonstrates

`compose --text` on a deep composition collapses through every level to a flat list of `## <id>` sections — one per atomic leaf, in composition order. The depth of the input is invisible in the output; an order-1 file produces N sections, an order-2 file produces N×M sections (or whatever the leaf count is), an order-23 file would produce one section per leaf — flat, no nesting, no intermediate-level headers leaking through.

## Steps that were run

1. **6 atomics** in the library:
   - `role-statement`, `safety-fence`, `tone-discipline` (re-used from earlier demos)
   - `format-rules`, `examples-discipline`, `file-citation-rules` (new for this demo)
2. **Two order-1 sub-prompts**:
   - `agent-foundations` = role-statement + safety-fence + tone-discipline
   - `output-discipline` = format-rules + examples-discipline + file-citation-rules
3. **One order-2 prompt**:
   - `senior-pair-deep` = agent-foundations + output-discipline
4. **`oovra compose --library library --text senior-pair-deep`** captures the prose output.

## Folder layout

```
03-deep-text-flattening/
├── library/
│   ├── role-statement.md          ← order 0
│   ├── safety-fence.md            ← order 0
│   ├── tone-discipline.md         ← order 0
│   ├── format-rules.md            ← order 0
│   ├── examples-discipline.md     ← order 0
│   ├── file-citation-rules.md     ← order 0
│   ├── agent-foundations.md       ← order 1, body_level 1
│   ├── output-discipline.md       ← order 1, body_level 1
│   └── senior-pair-deep.md        ← order 2, body_level 2
├── body-structure.md               ← delimiter-count proof of strict escalation
└── prose-output.md                 ← the --text rendering of senior-pair-deep
```

## Why the strict-escalation rule matters here

The order-2 file's body uses **3-tilde delimiters** (`~~~>>` / `~~~<<`) to wrap its two order-1 children. Each child's body uses **2-tilde delimiters** (`~~>>` / `~~<<`) to wrap its three order-0 grandchildren. When the parser splits the order-2 body, it scans for *exactly* 3-tilde delimiters — the inner 2-tilde delimiters are inert text from its perspective. Without strict monotonicity (the `body_level = max(input.order) + 1` rule), the outer parser could mis-split inside the children's bodies. See `body-structure.md` for the line-count proof.

## What the prose output reveals

- **6 H2 sections** — one per atomic leaf
- **The order is preserved**: agent-foundations' children come first (role-statement, safety-fence, tone-discipline), then output-discipline's children (format-rules, examples-discipline, file-citation-rules)
- **The order-1 sub-prompts are invisible** — their `agent-foundations` and `output-discipline` headers never appear in the prose form. They were structural scaffolding only.
- **No nesting** — the output is a flat sequence of H2 sections, regardless of the depth of composition

## The recursion in code

`render_for_paste()` in `src/render.rs`:

```rust
fn render_for_paste(element: &PromptElement) -> Result<String> {
    if element.header.is_atomic() {
        return Ok(format!("## {}\n\n{}", element.header.id, element.body.trim()));
    }
    let subs = decompose(element)?;
    let parts: Vec<String> = subs.iter().map(render_for_paste).collect::<Result<_>>()?;
    Ok(parts.join("\n\n"))
}
```

For atomic → emit one section. For composed → split body into sub-elements, recurse on each, join. There's no recursion-depth special case; an order-23 element flattens through `render_for_paste` calling itself 23 times deep, each call splitting one level and returning to the caller.

## Practical implication

You can compose elements at any depth — group atomics into role-specific sub-prompts, group sub-prompts into agent personas, group personas into agent families — and the prose form is always a clean flat list. The compositional hierarchy is purely organizational; it doesn't affect what the model sees.

## See also

- [reference/command-compose.md § Mode 2: --text](../../reference/command-compose.md#mode-2---text--print-clean-prose-to-stdout) — full reference for the prose-output mode (including the recursion contract)
- [reference/kind-and-delimiters.md](../../reference/kind-and-delimiters.md) — the strict escalation rule that makes deep parsing unambiguous
- [reference/command-decompose.md](../../reference/command-decompose.md) — `decompose` uses the same recursion in the other direction
- [demos/02-text-prose-output](../02-text-prose-output/) — simpler one-level case of the same mechanism
