---
id: handoff-after-tree
status: dropped
parent: "[[tree-form]]"
blocked_by: ["[[handoff-process]]"]
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-14T22:49:48+01:00
assignee: ai
owner: sevenuz
priority: 1
---
# Handoff note outdated after the tree form

- why: the handoff note (store item handoff) still describes tables, cards, the sort module and keys f/}; a new session would start from a wrong picture
- what: rewrite its form and code map parts: src/form/list.rs (all modules on the tree, detail box, focus), src/form/tree.rs (tree widget), card.rs and table.rs gone; module sort is inbox; config tree_depth, sort_who, form_auto_next
- done when: handoff matches commit 35de9a9 or later

dropped by the user (2026-11-14): tickets carry the state; chat mode in the skill
