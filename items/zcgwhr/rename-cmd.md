---
id: rename-cmd
status: task
parent: "[[zcgwhr]]"
created: 2026-11-15T18:19:20+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# Rename an item's id

- why: old ids like exqdqc are unexpressive and hard to type; the user wants to file with -p dashboard
- decided (user, 2026-11-15): `cake rename <old> <new>`; ids change only through it
- updates the file and its children's directory, children's parent, blocked_by everywhere, [[old]] links in all bodies
- then rename the six goal notes: exqdqc agents, zcgwhr cli, tjmbph stats, vvwfjv sync, jaddbh phone, zpatzq design
- done when: rename keeps the tree, blockers and links; tests