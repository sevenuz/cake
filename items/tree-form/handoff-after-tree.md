---
id: handoff-after-tree
status: task
parent: "[[tree-form]]"
blocked_by: ["[[handoff-process]]"]
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
priority: 1
---
# Handoff note outdated after the tree form

- why: the handoff note (store item handoff) still describes tables, cards, the sort module and keys f/}; a new session would start from a wrong picture
- what: rewrite its form and code map parts: src/form/list.rs (all modules on the tree, detail box, focus), src/form/tree.rs (tree widget), card.rs and table.rs gone; module sort is inbox; config tree_depth, sort_who, form_auto_next
- done when: handoff matches commit 35de9a9 or later
