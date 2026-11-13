---
id: claude-routine-triage-work
tags: [plan]
parent: "[[ai-workflow]]"
blocked_by: ["[[skill-triage-loop-ask]]"]
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Nightly Claude routine: triage and work while you sleep

- why: you want questions and finished work waiting in the morning,
  without starting a chat
- a routine is a Claude agent in the cloud on a schedule (e.g. 3:00); it
  sees only GitHub repos, so a store must be pushed (`cake sync`) to be in
- one routine, several repos; per repo it runs the skill's agent run
  (sync, answers, triage, work, sync)
- the repo list is kept by hand in the routine, edited with /schedule
  (user answer a)
- code changes go to a branch per item, never main; results come back as
  `review` items and questions
- done when: one night produced questions and a review item
