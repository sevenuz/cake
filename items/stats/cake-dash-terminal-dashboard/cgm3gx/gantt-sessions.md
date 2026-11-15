---
id: gantt-sessions
status: review
parent: "[[cgm3gx]]"
blocked_by: ["[[dash-overlays]]"]
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:54:12+01:00/2026-11-15T23:54:12+01:00
---
# Gantt G1: lanes per session

- you + each agent session, bars in the store's color, ··· where an agent waited on you, ▒ you in the form
- footer: max/avg parallel, agent wait on you; d day ↔ week
- done when: renders a test day as in [[dash-design]]

changed: g overlay, G1: you (chat █, form ▒), agents per store packed into lanes by overlap, 'waits on you' lane from questions that blocked work; day from the first hour with work, d week; footer max/avg at once and agent wait; commits: b59b075; try: cake, g
