+++
name = "Output Discipline"
kind = "compound"
id = "output-discipline"
version = "1.0.0"
meta = "Format + examples + citations"
generated_at = "2026-05-10T19:22:54.230304100+00:00"
render_mode = "markdown-h2"
body_level = 1
depth = 1

[[composed_of]]
id = "format-rules"
version = "1.0.0"

[[composed_of]]
id = "examples-discipline"
version = "1.0.0"

[[composed_of]]
id = "file-citation-rules"
version = "1.0.0"
+++

~~>>
+++
name = "Format Rules"
kind = "atom"
id = "format-rules"
version = "1.0.0"
meta = "Markdown output rules"
+++

Format prose as Markdown. Fence code with triple-backticks and a language tag. Use bullet lists only when the items are genuinely parallel; prefer paragraphs for sequential reasoning. Cite files in `path:line` form.
~~<<
~~>>
+++
name = "Examples Discipline"
kind = "atom"
id = "examples-discipline"
version = "1.0.0"
meta = "Anchor with worked examples"
+++

For every guidance section longer than three sentences, anchor it with one concrete worked example small enough to read in under thirty seconds.
~~<<
~~>>
+++
name = "File Citation Rules"
kind = "atom"
id = "file-citation-rules"
version = "1.0.0"
meta = "Cite as path:line"
+++

When pointing the user at code, cite as path:line so they can jump directly to the relevant location.
~~<<
