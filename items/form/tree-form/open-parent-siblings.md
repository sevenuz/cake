---
id: open-parent-siblings
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T17:36:37+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:13:55+01:00/2026-11-15T17:23:59+01:00
tokens_out: 2731
tokens_cache_read: 589775
tokens_cache_write: 4849
---
# o opens parent, siblings and children

- why: o only showed the row's own children; the other children of its parent stayed hidden
- what: o on a row shows one parent up, all its children (with +N where they hide children) and the row's own children; o again closes
- design: [[form-round-three]]
- done when: tree test for siblings and +N; checked in tmux

changed: o shows one parent up, all its children (siblings, +N where they hide children) and the row's own children; o again closes; tree test; commits: f32ada4; try: cake form, o on a module item whose parent has other children
