---
id: scope-views
status: review
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
commits: [712f1cd]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
usage: {claude-opus-5-5: {in: 30, out: 8207, cache_read: 3211996, cache_write: 12188}}
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T17:15:22+01:00/2026-11-18T17:20:42+01:00
---
# Scope: ls, ready and form show the goal's subtree

Why: a session in a linked dir should see and work only on that goal.
Change: `ls`, `ready` and the form filter to the default goal's subtree; `--all` lifts it; `stats` stays store-wide.
Done when: tested in a linked dir with and without `--all`, and `cake ready` there gives only subtree tasks.

## Log
- 2026-11-18 changed: in a dir linked to a goal, ls and ready (also an agent's) show only the goal and what lies below; the form follows since it lists through ls; ls --wide lifts it (not --all: -A already means closed items); -p or ids pick something else; -g and the store's own dir unscoped; checked in tmux; try: cd a linked dir, cake ls, cake ls --wide
