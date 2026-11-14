---
id: tree-sticky
status: task
parent: "[[tree-form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# Changed items stay visible until quit

- why: in sort, two rows set then h/l to answer and back: the rows are gone because they were saved and no longer match inbox; confusing
- what: the form keeps the set of ids changed this run; a module shows match ∪ changed; changed rows get a ✓ and stay editable
- saving stays immediate (store and undo unchanged)
- done when: scenario test: sort two items, h, l, both still shown with ✓
