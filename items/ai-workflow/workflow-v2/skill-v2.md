---
id: skill-v2
tags: [plan, skill]
parent: "[[workflow-v2]]"
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Skill for workflow v2

- why: agents must follow the new states, or the brakes do nothing
- statuses and assignee instead of tags; questions as blocking tasks
  for the user; comments appended; own notes as `note`, never inbox
- splitting: subtasks of your task as tasks with you as assignee, max
  depth 3; read `rule`/`goal` notes up the tree first ([[kqejfn]])
- ask only when a decision is hard to undo or changes the scope;
  otherwise decide and say so in the review note (what changed, commits,
  how to try)
- the agent run: loop over ready items until none or "waiting for
  review"; then a short summary
- done when: one agent run on a scratch store follows it without help
