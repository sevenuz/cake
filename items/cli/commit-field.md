---
id: commit-field
status: done
parent: "[[cli]]"
created: 2026-11-16T00:42:21+01:00
by: claude-code
commits: [4ed9ba7]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-16T02:25:18+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T00:48:43+01:00/2026-11-16T00:51:09+01:00
tokens_in: 9
tokens_out: 2664
tokens_cache_read: 1845954
tokens_cache_write: 3472
---
# Commits per item

- why: the user wants the git commits of a ticket on the ticket
- field commits (list of hashes); cake review <id> --commit a,b and cake edit --commit; shown in show, the form card and the dashboard card
- skill: agents pass --commit on review
- done when: round-trips, shown; tests

changed: commits list on items (markdown commits: [..], json commits); cake review|set|edit --commit a,b adds each once; show meta, form card and dashboard card list them; skill: agents pass --commit on review; try: cake show commit-field
