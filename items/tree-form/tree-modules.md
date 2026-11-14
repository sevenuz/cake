---
id: tree-modules
status: task
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[tree-sticky]]", "[[tree-edit-line]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# News, review, mine on the tree

- why: these modules are tables or cards today; one widget removes the parent-heading problem and keeps keys equal
- sort already runs on the tree (src/form/list.rs, done with tree-sticky and tree-edit-line); extend it with a kind per module
- what: news = new since last run; review = review items; mine = tasks assigned to the user; each with the edit line, save at once, ✓ sticky, u undo
- remove the table code (src/form/table.rs) once nothing uses it; also closes here-parent-description-useful
- done when: scenario test passes on all modules, screen dumps reviewed by the user
