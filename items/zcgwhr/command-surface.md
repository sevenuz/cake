---
id: command-surface
status: doing
tags: [cli]
parent: "[[zcgwhr]]"
created: 2026-11-14T01:11:22+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-14T01:11:22+01:00/
---
# Fewer commands: set, edit options, aliases

- why (user): 27 subcommands feel like too many; keep every function but
  reduce the surface to a minimum
- `set <status> <ids> [-m]` replaces do/note/review/done/drop; those stay
  as default aliases (done, drop, do, note, review)
- edit gets --assignee, --tag, --comment, --blocked-by, --parent; assign,
  tag, comment, block, unblock go
- ready becomes the default alias `ls --ready`; morning goes (cake / cake
  form); usage and migrate hidden from help
- remove the word morning and all German from the code
- done when: 14 visible commands, tests and skill adapted
