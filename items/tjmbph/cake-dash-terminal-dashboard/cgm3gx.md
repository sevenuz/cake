---
id: cgm3gx
tags: [tui]
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-10-27T13:11:05+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Gantt view of tasks

- timeline per task from `time` intervals and created/closed
- lanes per owner (`by`): user vs agents, parallel vs serial visible
- edges from parent and `blocked_by`
- critical path through `blocked_by` chains (longest tracked duration)
- filter with the normal selectors
