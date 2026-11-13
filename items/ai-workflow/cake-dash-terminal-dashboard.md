---
id: cake-dash-terminal-dashboard
tags: [plan]
parent: "[[ai-workflow]]"
blocked_by: ["[[deadline-field-asks-score]]"]
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Dashboard: one screen for what runs and what needs you

- why: `cake ls` shows everything as a tree; the questions "what runs,
  what waits on me, what is due" need several commands today
- `cake dash`, plain terminal output, one screen, ≤ 80 columns:
  - running: items in progress with their timers (you and agents)
  - waiting on you: open questions (count + top 3) and review items
  - due: hard deadlines in the next 7 days, overdue soft dates
  - time: tracked today and this week
- no new dependency, refreshes only when you run it again
- user (morning form): "approve but needs more planning"
- done when: shows the real store on one screen
