---
id: prio-inherit
status: done
parent: "[[cli]]"
created: 2026-11-15T18:47:52+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 10, out: 3737, cache_read: 1219312, cache_write: 5475}}
closed: 2026-11-15T19:48:42+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T18:47:54+01:00/2026-11-15T18:48:43+01:00
---
# Priority and deadline inherited from parents

- why: the user wants to rank the goals and have every task below follow, without setting prio on each task (instead of one active goal)
- decided (user, 2026-11-15): a task without its own prio takes the nearest ancestor's; same for the deadline; own values override
- score uses the effective values; --json adds `inherited`: {priority, deadline} when they come from above
- tree rows show an inherited prio/deadline dim
- rejected: adding the parent's score (hard to predict); rank only on goals
- done when: p1 on a goal lifts its tasks in cake ready; tests

changed: effective prio/deadline = own, else nearest ancestor's; score and cake ready use them; --json field inherited {priority, deadline}; tree rows show them dim and slanted; tests; commits: e353446; try: cake edit stats --prio 1, then cake ready
