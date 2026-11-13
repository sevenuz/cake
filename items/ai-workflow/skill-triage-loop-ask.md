---
id: skill-triage-loop-ask
status: done
tags: [plan]
parent: "[[ai-workflow]]"
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
closed: 2026-11-13T18:07:29+01:00
time:
  - 2026-11-13T17:50:11+01:00/2026-11-13T17:51:06+01:00
---
# Skill: triage loop and ask protocol

- names: title starts with topic (id = slug), no tool/parent/status
- triage steps: inbox query, mv/tag user items (never edit text),
  cleaned-up child, split rules, definition of ready
- ask protocol: ask item blocks its item, context + options a/b/c,
  answer via `done -m`, max 5 asks per run
- agent run steps usable by a Claude routine and a local model
- done when: dry run in a scratch store gives tree + asks ✓
- commit: cdef073
