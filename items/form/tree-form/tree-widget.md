---
id: tree-widget
status: done
parent: "[[tree-form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-14T12:10:33+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:06:00+01:00/2026-11-14T12:08:22+01:00
tokens_in: 20
tokens_out: 12507
tokens_cache_read: 993918
tokens_cache_write: 32105
---
# Tree widget shared by all modules

- why: every module needs the same tree with context rows; building it once keeps keys and looks the same everywhere
- what: input = all items + a match predicate; output = rows (match / context / collapsed `… ›`) with depth indent, `+N` hidden children, ✓ for changed items
- keys: j/k every row; open (`o <space>`) toggles body under the line and hidden children together; J/K and <c-d>/<c-u> as today
- config: `tree_depth` (default 2) in the commented config template
- done when: unit tests for depth limit, `… ›`, `+N`, reveal; screen dump shows one module on it

changed: new tree widget (src/form/tree.rs): matches + parents up to tree_depth, '… ›' when ancestors are cut, +N hidden children, ✓ changed items, open shows body under the line (tree lines continue) and reveals children dimmed; config tree_depth (default 2) in the template; template key list fixed (open o <space>, back <esc>, h/l modules); keys j/k/open are wired when the modules move (tree-modules); commits: 4aa97f3; try: CAKE_SCREENS=/tmp/s CAKE_STYLE=unicode cargo test --lib dump_screens_tree -- --ignored, then cat /tmp/s/tree-100-unicode.txt

accepted by the user in chat
