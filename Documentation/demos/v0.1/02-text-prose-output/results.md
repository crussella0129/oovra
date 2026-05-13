# Demo 2 — Results

## File sizes

| File | Size | Description |
|---|---|---|
| `pairing-agent.md` | 1709 bytes | the on-disk self-describing form |
| `prose-form-output.md` | 944 bytes | the `--text` output (with code-fence wrapper) |

## Section count in prose output

4 H2 sections — one per input element. Matches the 4 inputs in the parent's `composed_of` array.

## Conclusion

`oovra compose --text <id>` produces a clean Markdown prompt directly from any composed file, recursively flattening through every level of composition. The output is suitable for paste into a model with no further preprocessing. Metadata is intentionally stripped to keep the prompt minimal.
