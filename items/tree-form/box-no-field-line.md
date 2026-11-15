---
id: box-no-field-line
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 0, out: 2549, cache_read: 547084, cache_write: 4448}}
closed: 2026-11-15T17:33:19+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:14:01+01:00/2026-11-15T17:23:59+01:00
---
# Details box without the field line

- why: the field line `inbox · who – · prio – …` repeats the tree row as a table
- what: drop it; set fields stay after the title in the tree row
- design: [[form-round-three]]
- done when: no field line in any module's box; tests updated

changed: details box shows title, what/this run and the body, no field line; commits: f32ada4; try: cake form, any row
