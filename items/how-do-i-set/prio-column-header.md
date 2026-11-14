---
id: prio-column-header
status: review
parent: "[[how-do-i-set]]"
created: 2026-11-14T00:48:17+01:00
by: claude-code
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546, 81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-14T01:03:32+01:00/2026-11-14T01:04:45+01:00
---
# Priority column that says what it is

- why: the sort and mine tables head the priority column with "!", nobody can tell what it is or how to set it
- header "prio" instead of "!" (column grows from 2 to 5 chars, title 3 chars less)
- footer key hints in sort and mine list "p prio"
- done when: render test checks the header; 80-column test still passes

changed: the column says prio, mine shows p prio; commit d75f957; try: cake form sort
