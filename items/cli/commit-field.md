---
id: commit-field
status: task
parent: "[[cli]]"
created: 2026-11-16T00:42:21+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Commits per item

- why: the user wants the git commits of a ticket on the ticket
- field commits (list of hashes); cake review <id> --commit a,b and cake edit --commit; shown in show, the form card and the dashboard card
- skill: agents pass --commit on review
- done when: round-trips, shown; tests