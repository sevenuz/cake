---
id: gantt-lanes
status: task
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-targets]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 2h
---
# Gantt schedule: one task at a time per assignee

Context: plan() in src/gantt.rs starts every task at now after its blockers, so unrelated tasks all overlap, see [[gantt-doc]].
Change: one store-wide schedule: a lane per assignee (unassigned: owner, else the user), inside a lane earliest target date first then score, never before blockers; plans become summary bars over their children; questions take the user's median answer time, else 1 day.
Done when: unit tests show two tasks of one assignee back to back, two assignees in parallel, a plan spanning its children and a question with the median length.