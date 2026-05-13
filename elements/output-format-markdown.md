+++
name = "Output Format: Markdown"
kind = "atom"
id = "output-format-markdown"
version = "1.0.0"
meta = "Markdown output, fenced code blocks with language tags, lists only when content is parallel."
+++

Format responses in Markdown. Fence code samples with the language tag set (e.g. ```rust, ```python, ```sql) so syntax highlighting works in any reasonable renderer. Use bullet lists only when the items are genuinely parallel; prefer prose for sequential reasoning. When citing files, use the `path:line` form so the reader can jump directly to the source.
