---
id: skill-blocked-by
status: review
parent: "[[agents]]"
created: 2026-11-20T22:11:11+01:00
by: claude-code
commits: [2c09135]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 20m
time:
  - 2026-11-20T22:11:14+01:00/2026-11-20T22:12:29+01:00
---
# Triage proposes missing blocked_by links

Context: the gantt schedule (see [[gantt-doc]]) only shows a real path when tasks that build on each other are linked with blocked_by; the user's tasks rarely are.
Why: without order the critical path is one task long and says nothing.
Change: in the skill's gardening step, triage looks at the open tasks of each plan and asks in the summary about missing order ("Does fix-stat wait for fix-inode?"), never setting blocked_by on the user's items itself. Decided by the user on 2026-11-20.
Done when: skill/SKILL.md says so in the garden step.

## Log
- 2026-11-20 changed: the garden step in the skill asks 'Does <a> wait for <b>?' in the triage summary for a plan's tasks without order, never sets it on the user's items; try: read the garden step in skill/SKILL.md
