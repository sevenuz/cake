---
id: tree-rows-design
tags: [doc]
parent: "[[tree-form]]"
created: 2026-11-14T21:11:23+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
owner: sevenuz
---
# Tree rows: set values after the title, no table

- why: the table right of the tree is wide, has its own header line and looks odd next to tree lines
- decided (user, 2026-11-14): no columns; after the title only the values that are set, joined by ` · `, colored like the `:` hint
- order: what (news) · status (only review, done, dropped, doing/running) · who · prio pN · due · deadline
- colors stay: late dates red, near dates yellow, prio 1 red, prio 2 yellow, who in the hint color, me plain; context rows all dim
- status word left out for inbox/task/note: the status mark before the title shows it
- the edit line under the hovered row ([[tree-edit-line]]) is dropped: the details box shows all fields
- the end screen of the form becomes the whole tree with the details box below ([[last-tab-whole-tree]])
- example: `› ○ my own task  me · p1 · due 20.11.`
