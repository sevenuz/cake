---
id: last-tab-whole-tree
status: doing
parent: "[[tree-form]]"
created: 2026-11-14T20:51:28+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
usage: {claude-opus-5-5: {in: 10, out: 4917, cache_read: 764562, cache_write: 8756}}
closed: 2026-11-14T21:29:07+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T21:13:31+01:00/2026-11-14T21:15:01+01:00
  - 2026-11-14T22:45:08+01:00/
---
# Last tab: whole tree with details box at the bottom

- why: the last tab should show everything, like the other modules show their part
- whole tree, scrollable when long; details of the hovered item in the box below
- the box stays at the bottom of the terminal, the tree scrolls above it
- done when: a tree longer than the pane scrolls with the box fixed; scenario test

changed: after the last module the form opens the tab ALL instead of the end card: every open item of the store as one tree, details box below; long trees scroll with the box fixed at the bottom; c/dd/w/p and :due etc. work there; the title line says what this run decided; l there says it is the last tab, h goes back, :q quits; empty store: the old end card; scenario test; commits: 9350db8; try: cake form, l past the last module

comment (claude-code, 2026-11-14): q quits on the last tab (no :q needed there); ALL shows in the header from the start, not only once you are on it
