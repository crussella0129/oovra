# v0.1 — Version Reports

Three planning/assessment reports written while v0.1 was the current state. Together they survey the project from three angles: looking forward, looking sideways, and looking at itself.

| Report | Purpose | When to read |
|---|---|---|
| [`v0.2-scoping.md`](./v0.2-scoping.md) | What's missing from v0.1 that's worth adding to v0.2; what's already replicable; deep dive on `compare` improvements (sequence-aware diff, body-content diff, broader header tracking) | Before starting any v0.2 work — this is the scope contract |
| [`oovra-server-feasibility.md`](./oovra-server-feasibility.md) | Design + feasibility for an HTTP service that turns Oovra into a network-accessible prompt library for multi-machine deployments (OpenClaw fleets, Ollama setups, team libraries) | If/when deciding whether to build `oovra-server` as part of v0.2 or defer to v0.3 |
| [`v0.1-efficacy-rating.md`](./v0.1-efficacy-rating.md) | Honest assessment of what v0.1 is good for, who'd use it, where to share it online, base-model fit, competitor landscape, and what limits adoption today | Before promoting the tool, deciding feature priorities, or pitching it to a community |

## What this folder is

These are **v0.1-era planning documents**. They were captured while v0.1 was the current shipping state — before any v0.2 code was written. The numeric ratings, scope estimates, and community-fit assessments reflect what was knowable at that moment.

When v0.2 ships, this folder should be left in place as a historical record. A fresh `v0.2/version reports/` folder can be created for the next planning cycle.

## Reading order

If you're approaching the project fresh and want context on where it's headed:

1. **Start with [`v0.1-efficacy-rating.md`](./v0.1-efficacy-rating.md)** — answers "is this worth investing in?" with strengths, limitations, and target users.
2. **Then [`v0.2-scoping.md`](./v0.2-scoping.md)** — answers "what should we build next?" with a tiered priority list.
3. **Optionally [`oovra-server-feasibility.md`](./oovra-server-feasibility.md)** — answers "should we build the server feature, and if so when?" — deferred unless concrete demand emerges.

## Snapshot date

All three reports were generated against **Oovra v0.1.0**. See the commit history for exact dates.
