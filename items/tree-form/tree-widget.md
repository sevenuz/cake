---
id: tree-widget
status: task
parent: "[[tree-form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# Tree widget shared by all modules

- why: every module needs the same tree with context rows; building it once keeps keys and looks the same everywhere
- what: input = all items + a match predicate; output = rows (match / context / collapsed `… ›`) with depth indent, `+N` hidden children, ✓ for changed items
- keys: j/k every row; open (`o <space>`) toggles body under the line and hidden children together; J/K and <c-d>/<c-u> as today
- config: `tree_depth` (default 2) in the commented config template
- done when: unit tests for depth limit, `… ›`, `+N`, reveal; screen dump shows one module on it
