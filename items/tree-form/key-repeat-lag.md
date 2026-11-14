---
id: key-repeat-lag
status: review
parent: "[[tree-form]]"
created: 2026-11-14T21:34:43+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T22:39:50+01:00/2026-11-14T22:41:21+01:00
---
# Holding j lags and jumps a random amount

- why: holding j moves late and by a random amount; movement feels out of control
- likely cause: every key reloads all items with a `cake ls` subprocess before drawing; key repeat queues up behind it
- fix: reload only after a change (save, undo, editor, timer), moves only redraw
- done when: holding j moves one row per repeat, stops when released

changed: the form keeps the item list per store and runs cake ls again only when the items dir changed on disk (entry count or newest mtime): own saves, agents and sync still show at once; moving the cursor is a redraw only; 20 j in tmux: done within 0.3 s, before still several rows behind; commits: 5ab4456; try: hold j in the last tab
