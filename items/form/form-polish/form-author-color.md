---
id: form-author-color
status: review
parent: "[[form-polish]]"
created: 2026-11-15T18:21:10+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 24, out: 5703, cache_read: 1363684, cache_write: 13478}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T20:38:16+01:00/2026-11-15T20:40:41+01:00
---
# Your own items in their own color in the tree

- why: the user wants to see how agents restructure and where they put subtasks and details under the user's own (often vague) items
- what: in the form's tree, items written by the user (`by` = you) get one color, agent items another; marks and fields unchanged
- open: which side is bright (your items plain/bright, agent items in the hint color?); decide in review
- the user's text stays untouched anyway (skill rule)
- done when: ALL shows both colors; test on the rendered rows

changed: ALL colors titles of items you wrote (by = you) in a new theme key mine (default orange #fe8019); your context rows (parents, notes) orange but dimmed; agent items unchanged; ? legend line; decided: your side bright, since agents write most items; commits: 5c6977e; try: cake, l to ALL; change the color in ~/.config/cake/theme.hjson mine: ...
