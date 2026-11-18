---
id: box-no-field-line
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T17:36:37+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:14:01+01:00/2026-11-15T17:23:59+01:00
tokens_out: 2549
tokens_cache_read: 547084
tokens_cache_write: 4448
---
# Details box without the field line

- why: the field line `inbox · who – · prio – …` repeats the tree row as a table
- what: drop it; set fields stay after the title in the tree row
- design: [[form-round-three]]
- done when: no field line in any module's box; tests updated

changed: details box shows title, what/this run and the body, no field line; commits: f32ada4; try: cake form, any row
