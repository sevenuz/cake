---
id: tree-inline-fields
status: done
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-14T21:28:50+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T21:11:35+01:00/2026-11-14T21:13:03+01:00
tokens_in: 8
tokens_out: 4986
tokens_cache_read: 570290
tokens_cache_write: 8316
---
# Tree rows: colored fields instead of the mini table

- why: the mini table in the tree is wide and odd next to the tree lines
- user idea: after the title, fields separated and color coded like the `:` hint (status · who · prio · due)
- at least: the table takes much less width
- done when: tree rows fit narrow panes and stay readable; decided with the user ([[tree-edit-line]] comment)

changed: the table and its header line are gone; after each title only the values that are set: status (only running/doing/review/done/dropped, the mark shows the rest) · who · pN · due · deadline, colored like before (late red, near yellow, p1 red); the title is cut so the values fit; design in [[tree-rows-design]]; test tree_fields_show_only_set_values; commits: 6206907; try: cake form mine
