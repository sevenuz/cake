---
id: tree-sticky
status: review
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
usage: {claude-opus-5-5: {in: 1, out: 386, cache_read: 71621, cache_write: 479}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:16:16+01:00/2026-11-14T12:16:16+01:00
---
# Changed items stay visible until quit

- why: in sort, two rows set then h/l to answer and back: the rows are gone because they were saved and no longer match inbox; confusing
- what: the form keeps the set of ids changed this run; a module shows match ∪ changed; changed rows get a ✓ and stay editable
- saving stays immediate (store and undo unchanged)
- done when: scenario test: sort two items, h, l, both still shown with ✓

changed: sort runs on the tree (src/form/list.rs); every key saves at once; changed items stay with ✓ until the form quits, also after h/l (kept in the form state per store); u restores the item as it was before the run; <cr> goes on (nothing unsaved anymore); commits: 21ed863; try: cake, sort two items, h then l: both still there with ✓; u on one puts it back
