---
id: command-surface
status: review
tags: [cli]
parent: "[[zcgwhr]]"
created: 2026-11-14T01:11:22+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
usage: {claude-opus-5-5: {in: 48, out: 19181, cache_read: 20867867, cache_write: 36740}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-14T01:11:22+01:00/2026-11-14T01:16:57+01:00
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

changed: 14 commands (set, edit --tag/--blocked-by/--parent/--comment, aliases done/drop/do/note/review/ready), morning and German removed, usage/migrate hidden; commit 4e7ee97; try: cake --help
