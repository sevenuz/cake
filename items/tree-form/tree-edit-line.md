---
id: tree-edit-line
status: task
parent: "[[tree-form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# Edit line under the hovered entry

- why: values are set in table columns now; the user wants to see and set them right under the item they are on
- what: under the hovered editable row one line `status · who:ai · prio – · due – · deadline –`; current values filled in
- keys: the existing value keys (t/n/dd/x, w, p, :due, :deadline) write into the line; moving between fields: see question on h/l
- sort: who starts with config `sort_who` (default "ai") when an item becomes a task
- done when: values set from the line land in the store; sort_who in config template; test with sort_who "-"
