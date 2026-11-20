---
id: gantt-targets
status: doing
parent: "[[gantt-rework]]"
created: 2026-11-20T21:56:28+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-20T22:12:32+01:00/
---
# Gantt targets: every open item with a deadline or due

Context: the critical path view only knows goal notes with a deadline (goals_with_deadline in src/gantt.rs), see [[gantt-doc]].
Change: collect every open item of the store with a deadline or due; an item inside another target's subtree becomes a marker of the outer one; sort by date, goals first on equal dates.
Done when: unit tests cover a goal with a nested task deadline, a lone task with a due, an inbox item with a date and a note due far ahead.