---
id: skill-triage-loop-ask
tags: [plan]
parent: "[[ai-workflow]]"
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Skill: triage loop and ask protocol

- triage steps: read inbox, mv/tag user items (never edit text),
  cleaned-up child, split rules, definition of ready
- ask protocol: format (context + options), blocking, max N asks per run
- run prompt usable by a Claude routine and a local model agent
- done when: dry run on 5 real inbox thoughts gives sensible tree + asks
