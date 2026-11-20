---
id: gantt-targets
status: done
parent: "[[gantt-rework]]"
created: 2026-11-20T21:56:28+01:00
by: claude-code
commits: [1f4b9fe]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-20T23:06:58+01:00
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-20T22:12:32+01:00/2026-11-20T22:14:45+01:00
tokens_in: 24
tokens_out: 5904
tokens_cache_read: 1374544
tokens_cache_write: 13744
---
# Gantt targets: every open item with a deadline or due

Context: the critical path view only knows goal notes with a deadline (goals_with_deadline in src/gantt.rs), see [[gantt-doc]].
Change: collect every open item of the store with a deadline or due; an item inside another target's subtree becomes a marker of the outer one; sort by date, goals first on equal dates.
Done when: unit tests cover a goal with a nested task deadline, a lone task with a due, an inbox item with a date and a note due far ahead.

## Log
- 2026-11-20 changed: targets() in the gantt module replaces goals_with_deadline: every open item or note with a deadline or due, nested targets as inner markers of the outermost; the critical path view and the focus warning use it (view still one target per page until gantt-view); try: g, v to the critical path in the phd store, j/k now also reaches submit-register-free targets like lone tasks
