---
id: tree-tags-zz
status: review
parent: "[[form-polish]]"
created: 2026-11-15T22:56:23+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 18, out: 6945, cache_read: 1963515, cache_write: 14436}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T22:56:23+01:00/2026-11-15T22:58:48+01:00
---
# Tree: tags on rows, in the card, and zz

- why: tags were never shown in the form
- card: tags in the dim meta line (#doc #goal)
- tree: tags dim after the title, on by default, # toggles
- zz centers the view on the cursor
- done when: tags visible in tree and card, # and zz work; tests

changed: tags dim after the title in every tree, on by default, # toggles, dropped first when the row is too narrow; card shows #tags in its first line; zz centers on the cursor; the tree view now stays put while the cursor is in view (vim-like) instead of always pinning it to the bottom; commits: f572a51; try: cake, l to ALL, # and zz
