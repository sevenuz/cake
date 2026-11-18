---
id: fields-stats
status: task
parent: "[[fields]]"
blocked_by: ["[[fields-config]]"]
created: 2026-11-18T16:46:24+01:00
by: claude-code
commits: [4c878d6, 20174b9]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 4h
time:
  - 2026-11-18T22:00:00+01:00/2026-11-18T22:08:22+01:00
---
# Fields: declarative stats sections and merge_by

Why: the agents and footprint sections are AI code in the core.
Change: `stats.sections` rows (sum, avg, per_done, top, heat) with the period share; `stats.merge_by` replaces the session merge; the built-in agents and footprint code goes; the example config reproduces today's sections.
Done when: the example config shows the same numbers as today on this store, tests pass.

## Log
- 2026-11-18 comment (claude-code): progress (claude-code): code done and tested (4c878d6, 20174b9); live switch-over waits for the user's ok: release build, Stop hook to hooks/claude-code.py, stats block into the user's config, migrate --only usage on the cake store (69 items)
