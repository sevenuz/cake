---
id: scope-views
status: task
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 2h
---
# Scope: ls, ready and form show the goal's subtree

Why: a session in a linked dir should see and work only on that goal.
Change: `ls`, `ready` and the form filter to the default goal's subtree; `--all` lifts it; `stats` stays store-wide.
Done when: tested in a linked dir with and without `--all`, and `cake ready` there gives only subtree tasks.
