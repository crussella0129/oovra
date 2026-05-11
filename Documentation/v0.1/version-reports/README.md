# v0.1 — Version Reports

Four planning/assessment reports written while v0.1 was the current state. Together they survey the project from four angles: looking forward, looking sideways, looking at itself, and looking at what's structurally holding it back.

| Report | Purpose | When to read |
|---|---|---|
| [`v0.2-scoping.md`](./v0.2-scoping.md) | What's missing from v0.1 that's worth adding to v0.2; what's already replicable; deep dive on `compare` improvements (sequence-aware diff, body-content diff, broader header tracking) | Before starting any v0.2 work — this is the scope contract |
| [`oovra-server-feasibility.md`](./oovra-server-feasibility.md) | Design + feasibility for an HTTP service that turns Oovra into a network-accessible prompt library for multi-machine deployments (OpenClaw fleets, Ollama setups, team libraries) | If/when deciding whether to build `oovra-server` as part of v0.2 or defer to v0.3 |
| [`v0.1-efficacy-rating.md`](./v0.1-efficacy-rating.md) | Honest assessment of what v0.1 is good for, who'd use it, where to share it online, base-model fit, competitor landscape, and what limits adoption today (rating: 7.4/10) | Before promoting the tool, deciding feature priorities, or pitching it to a community |
| [`v0.1-ceiling-analysis.md`](./v0.1-ceiling-analysis.md) | Follow-up to the efficacy rating: 12 structural limitations that cap Oovra's ceiling regardless of execution quality. What it would take to push to 10/10. Includes subjective probabilities on whether the underlying thesis is right. | When deciding strategy: are we executing on the right thing? Do v0.2 priorities need revision? |

## What this folder is

These are **v0.1-era planning documents**. They were captured while v0.1 was the current shipping state — before any v0.2 code was written. The numeric ratings, scope estimates, community-fit assessments, and ceiling-blocker analyses reflect what was knowable at that moment.

When v0.2 ships, this folder should be left in place as a historical record. A fresh `v0.2/version-reports/` folder can be created for the next planning cycle.

## Reading order

If you're approaching the project fresh and want context on where it's headed:

1. **Start with [`v0.1-efficacy-rating.md`](./v0.1-efficacy-rating.md)** — answers "is this worth investing in?" with strengths, limitations, target users, and a 7.4/10 numeric rating.
2. **Then [`v0.1-ceiling-analysis.md`](./v0.1-ceiling-analysis.md)** — answers "what's keeping this from being a perfect tool?" Honest, opinionated. Surfaces structural limits the efficacy rating doesn't capture.
3. **Then [`v0.2-scoping.md`](./v0.2-scoping.md)** — answers "what should we build next?" with a tiered priority list. The ceiling-analysis suggests three revisions worth considering: adding templating, `oovra init` + tutorial, and integration docs.
4. **Optionally [`oovra-server-feasibility.md`](./oovra-server-feasibility.md)** — answers "should we build the server feature, and if so when?" — deferred unless concrete demand emerges.

## How the four reports relate

- **Efficacy rating** rates dimensions (correctness, ergonomics, docs, etc.) and gives a numeric score
- **Ceiling analysis** explains why the score has a structural cap — what's blocking a 10/10 that the dimension-rating doesn't see
- **v0.2 scoping** proposes the next round of features; the ceiling analysis suggests three additions to that scope
- **Server feasibility** is a separate strategic question (build a service layer?) — recommends deferral

The four together support a strategic decision: **build v0.2 with the ceiling-analysis revisions, then measure adoption.** If adoption signals come in, the manual-curation thesis is validated and the path forward is clear. If they don't, the right move is a pivot rather than v0.3.

## Snapshot date

All four reports were generated against **Oovra v0.1.0**. See the commit history for exact dates.
