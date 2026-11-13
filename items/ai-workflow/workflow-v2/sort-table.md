---
id: sort-table
status: review
tags: [plan, tui]
parent: "[[workflow-v2]]"
blocked_by: ["[[form-modules]]"]
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
---
# Sort table: inbox and assignment in one view

- why: sorting many dumped ideas one per screen is slow
- one row per inbox item (and tasks without assignee): kind, title,
  who, due, prio; j/k move, n note, t task (who = you), x drop, m move to
  another store, w who, d due, D deadline, p prio, o open the text
- one key decides and moves to the next row; nothing is saved until
  enter; undecided rows stay in the inbox
- done when: 10 dumped ideas sorted in under a minute

changed: inbox and tasks without assignee in one table: n/t/x, w who, d/D dates, p prio, m move store, A subtree, o open, enter saves; commit 5aeb0b5; try: cake form sort
