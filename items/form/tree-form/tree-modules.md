---
id: tree-modules
status: done
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[tree-sticky]]", "[[tree-edit-line]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 2, out: 211, cache_read: 68242, cache_write: 419}}
closed: 2026-11-14T20:32:44+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:32:04+01:00/2026-11-14T12:32:04+01:00
---
# News, review, mine on the tree

- why: these modules are tables or cards today; one widget removes the parent-heading problem and keeps keys equal
- sort already runs on the tree (src/form/list.rs, done with tree-sticky and tree-edit-line); extend it with a kind per module
- what: news = new since last run; review = review items; mine = tasks assigned to the user; each with the edit line, save at once, ✓ sticky, u undo
- remove the table code (src/form/table.rs) once nothing uses it; also closes here-parent-description-useful
- done when: scenario test passes on all modules, screen dumps reviewed by the user

changed: news, sort, review, mine on the tree too, no box: tree fills the screen and scrolls; edit line in news/sort/mine; review c/dd/i and <cr> card; news w hold, dd drop; mine s/c/dd/w/p/:due; table and card code, overview, f hints and } removed; commits: 0b0a56d; try: cake
