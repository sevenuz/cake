---
id: dump-still-layout
status: review
parent: "[[tree-form]]"
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:14:43+01:00/2026-11-15T17:23:59+01:00
---
# Dump and end screens in the new layout

- why: the dump module (ideas into the inbox), the "nothing here" and end screens still use the old card layout; they look different from the tree modules
- what: same look: title line in the module color, box below, same key line
- done when: screen dumps of dump, empty module and end screen match the tree modules

changed: dump, empty module and end screens use the tree layout: title line on top, box fixed at the bottom; a adds an item from the empty screens too; commits: f32ada4; try: cake form, h/l to an empty module; dump
