---
id: tree-overlay
status: review
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[jk-in-navigation]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:32:04+01:00/2026-11-14T12:32:04+01:00
---
# Subtree overlay on A

- why: the module only shows a slice; to read parents, siblings and their bodies the user needs the whole tree
- what: `O` opens an overlay with the full tree from the root above the hovered item, same widget, all rows readable
- keys: j/k, open (`o <space>`) body and children, `<cr>` close and jump to the item if the module has it, `<esc>`/`q` close; `A` stays sort's decision for the rows below
- read only
- done when: scenario test opens O, reads a sibling's body, closes

changed: O shows the whole tree around the row (from its top item), read only; j/k, o, <cr> goes to the item, <esc>/q close; key full_tree in the config; commits: 0b0a56d; try: cake form answer, O
