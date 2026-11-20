---
id: gantt-select
status: doing
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-view]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-20T22:22:07+01:00/
---
# Gantt view: select rows and act on them

Context: after [[gantt-view]] the critical path view lists tasks of all targets; the items view of the gantt already has a card, enter and e for the selected row.
Change: j/k move over the task rows of all blocks (scrolling), the card of the selected item shows under the gantt, enter opens the tree at it, e edits it.
Done when: a dashboard test moves the selection across a block boundary and sees the card of the right item.