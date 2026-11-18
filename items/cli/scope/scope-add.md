---
id: scope-add
status: task
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h
---
# Scope: captures land under the default goal

Why: a dump in ~/git/gekkofs should already sit under `gekkofs`.
Change: `add` and form capture without `-p` use the default; `-p -` means top level; output shows `→ <goal>`; missing goal → top level with a warning.
Done when: tested from the dir, a subdir, with `-p -`, `-g` and a missing goal.
