---
id: gantt-lanes
status: review
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-targets]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
commits: [f5ab122]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-20T22:15:28+01:00/2026-11-20T22:18:40+01:00
---
# Gantt schedule: user in a row, agents in parallel

Context: plan() in src/gantt.rs starts every task at now after its blockers, so unrelated tasks all overlap, see [[gantt-doc]].
Change: one store-wide schedule. The user's lane runs their tasks, unassigned tasks and the review steps of agent tasks one after another, earliest target date first then score, never before blockers. Agent tasks run in parallel, at most max_review at once (running or waiting for review); each ends with a review step in the user's lane (median review-to-done time, else 15 min). Plans are summary bars over their children; questions take the user's median answer time, else 1 day.
Done when: unit tests show two user tasks back to back, agent tasks in parallel capped by max_review, the review step in the user's lane, a plan spanning its children and a question with the median length.

## Log
- 2026-11-20 changed: one schedule for the whole store: your tasks, unassigned ones and reviews of agent tasks one after another (nearest target date first, then score); agent tasks in parallel, at most max_review running or in review; review step = median review time from history, else 15 min; questions (your tasks that block an agent's task) = median answer time, else 1 day; plans span their children; on the phd store: 55h of your work queued before submit-upload. The drawing is still the old one until gantt-view. try: cargo test gantt
