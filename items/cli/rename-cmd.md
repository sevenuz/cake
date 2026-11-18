---
id: rename-cmd
status: done
parent: "[[cli]]"
created: 2026-11-15T18:19:20+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T19:48:42+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T18:19:20+01:00/2026-11-15T18:21:10+01:00
tokens_in: 14
tokens_out: 3729
tokens_cache_read: 1643261
tokens_cache_write: 6405
---
# Rename an item's id

- why: old ids like exqdqc are unexpressive and hard to type; the user wants to file with -p dashboard
- decided (user, 2026-11-15): `cake rename <old> <new>`; ids change only through it
- updates the file and its children's directory, children's parent, blocked_by everywhere, [[old]] links in all bodies
- then rename the six goal notes: exqdqc agents, zcgwhr cli, tjmbph stats, vvwfjv sync, jaddbh phone, zpatzq design
- done when: rename keeps the tree, blockers and links; tests

changed: cake rename <id> <new> (prefix ok): file and children's directory move, children's parent, blocked_by everywhere and [[old]] links in all bodies follow; skill documents it (only when the user asks); renamed exqdqc→agents, zcgwhr→cli, tjmbph→stats, vvwfjv→sync, jaddbh→phone, zpatzq→design; tests store + CLI; commits: a3d59b3; try: cake ls, cake add x -p stats
