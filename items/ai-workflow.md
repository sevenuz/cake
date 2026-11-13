---
id: ai-workflow
status: task
tags: [plan]
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
---
# AI workflow: capture, triage, ask, work

- capture: brain dump into the inbox (top-level, no status, by user)
- triage (agent): mv into tree, planned children with `done when`, split
  when > 5 bullets or > 1 `done when`, depth ≤ 3
- ask: one question per item, tag `ask`, blocks the item it is about,
  body = short context + options a/b/c; answer goes into the ask item
- morning form (user): paper todos, then asks/unspecified by score
- work (agent): ready approved items; new ask when stuck, next item
- cake itself never calls an AI: agents drive it via CLI + skill
  (Claude routine now, local model later)
- done when: phases below done, one week used for real
