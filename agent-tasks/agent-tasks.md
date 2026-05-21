# agent-tasks.md

Active task list for the agentic build work, populated from the
current sprint's `build-plan.md`. Tasks are executed in order;
completed entries are removed from this file and appended to
`completed-tasks.md`.

This file is independent of any single sprint — it persists across
sprints and reflects the work currently in flight.

## (empty — between sprints)

Sprint s6 (egui Panel-alias migration) is closed; the prior
correlated-list-items fix also landed. Roadmap-next options:
- **Syntax-highlighted editor + Markdown preview** (egui_code_editor
  + egui_commonmark; visible GUI improvement, no external tooling).
- **WASM filesystem shim + Trunk pipeline** (the web build's
  missing filesystem layer; needs Trunk + browser testing).
- **Compound recipe editing** (edit a compound's composed_of in
  the GUI — deferred since s2).
- **Cross-olib compare** (deferred since s4).
