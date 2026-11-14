---
id: tree-overlay
status: task
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[jk-in-navigation]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# Subtree overlay on A

- why: the module only shows a slice; to read parents, siblings and their bodies the user needs the whole tree
- what: `A` opens an overlay with the full tree from the root above the hovered item, same widget, all rows readable
- keys: j/k, o fold, `<space>` body, `<cr>` close and jump to the item if the module has it, `<esc>`/`q` close
- read only
- done when: scenario test opens A, reads a sibling's body, closes
