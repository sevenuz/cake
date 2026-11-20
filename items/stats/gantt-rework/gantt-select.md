---
id: gantt-select
status: done
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-view]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
commits: [212bc1a]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-20T23:07:06+01:00
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-20T22:22:07+01:00/2026-11-20T22:23:59+01:00
tokens_in: 10
tokens_out: 4914
tokens_cache_read: 844678
tokens_cache_write: 8338
---
# Gantt view: select rows and act on them

Context: after [[gantt-view]] the critical path view lists tasks of all targets; the items view of the gantt already has a card, enter and e for the selected row.
Change: j/k move over the task rows of all blocks (scrolling), the card of the selected item shows under the gantt, enter opens the tree at it, e edits it.
Done when: a dashboard test moves the selection across a block boundary and sees the card of the right item.

## Log
- 2026-11-20 changed: in the deadlines view j/k select task rows across all blocks (the view scrolls to keep it in sight), the selected item's card shows below, enter opens the tree at it, e edits it, as in the items view; try: g, v v v, j a few times, enter
