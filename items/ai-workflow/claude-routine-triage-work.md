---
id: claude-routine-triage-work
tags: [plan]
parent: "[[ai-workflow]]"
blocked_by: ["[[skill-triage-loop-ask]]"]
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Claude routine for triage and work

- scheduled routine runs the skill prompt: triage inbox, then ready
  items, opens asks, ends in review
- needs pushed store branch (cake sync in routine only)
- code work on a branch per item, no push to main
- later: same prompt with a local model agent
- done when: one overnight run produced asks + a review item
