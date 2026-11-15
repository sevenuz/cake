---
id: prio-inherit
status: doing
parent: "[[cli]]"
created: 2026-11-15T18:47:52+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T18:47:54+01:00/
---
# Priority and deadline inherited from parents

- why: the user wants to rank the goals and have every task below follow, without setting prio on each task (instead of one active goal)
- decided (user, 2026-11-15): a task without its own prio takes the nearest ancestor's; same for the deadline; own values override
- score uses the effective values; --json adds `inherited`: {priority, deadline} when they come from above
- tree rows show an inherited prio/deadline dim
- rejected: adding the parent's score (hard to predict); rank only on goals
- done when: p1 on a goal lifts its tasks in cake ready; tests