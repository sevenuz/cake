---
id: skill-triage-loop-ask
status: done
tags: [plan]
parent: "[[ai-workflow]]"
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-13T18:07:29+01:00
time:
  - 2026-11-13T17:50:11+01:00/2026-11-13T17:51:06+01:00
tokens_in: 8
tokens_out: 5688
tokens_cache_read: 504361
tokens_cache_write: 6477
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
