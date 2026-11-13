---
id: agent-ready-rules
status: task
tags: [plan]
parent: "[[workflow-v2]]"
blocked_by: ["[[statuses-and-assignee]]"]
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
---
# What an agent may work on, and the review brake

- why: the AI should work until it is really blocked, but never produce
  more than you can review
- ready for an agent = task assigned to it or to `ai`, nothing open in
  blocked_by (review counts as open), no open children
- a parent whose children are all done goes to review by itself
- store config `.cake/config.hjson`: `people`, `agents` (who `w` cycles
  through, unknown names refused) and `max_review` (default 8, -1 off):
  while that many reviews wait for you, `cake ready` gives agents
  nothing and `cake start` refuses with "waiting for review"
- done when: tests for siblings continuing, a dependent waiting on a
  review, the parent going to review, max_review 8 and -1
