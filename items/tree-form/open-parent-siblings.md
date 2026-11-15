---
id: open-parent-siblings
status: task
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# o opens parent, siblings and children

- why: o only showed the row's own children; the other children of its parent stayed hidden
- what: o on a row shows one parent up, all its children (with +N where they hide children) and the row's own children; o again closes
- design: [[form-round-three]]
- done when: tree test for siblings and +N; checked in tmux