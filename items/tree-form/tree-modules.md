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
# Sort, news, review, mine on the tree

- why: these modules are tables today; one widget removes the parent-heading problem and keeps keys equal
- what: sort = inbox items; news = new since last run; review = review items; mine = tasks assigned to the user; each with its edit line and existing keys
- remove the table code that is no longer used; also closes here-parent-description-useful
- done when: scenario test passes on all modules, screen dumps reviewed by the user
