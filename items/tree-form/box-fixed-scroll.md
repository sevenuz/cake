---
id: box-fixed-scroll
status: task
parent: "[[tree-form]]"
created: 2026-11-14T21:34:43+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Detail box: fixed height, own scrolling, contained keys

- why: the detail box changes height with every row, and gg in the box jumped to the top of the tree
- box height fixed (see question), text scrolls inside on overflow
- keys act on the focused pane only (tab switches): j/k gg G c-d c-u move in the tree or in the box, never both
- done when: moving through rows never resizes the box; gg/G in the box go to its top/bottom; tests
