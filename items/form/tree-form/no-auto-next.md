---
id: no-auto-next
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T17:36:37+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:12:59+01:00/2026-11-15T17:23:59+01:00
tokens_in: 6
tokens_out: 4391
tokens_cache_read: 967456
tokens_cache_write: 7173
---
# Module stays after its last item, with a done mark

- why: the automatic switch to the next module is not noticeable and confuses
- what: no switch, config form_auto_next removed; header tab ✓, note "all done here · l next module"
- design: [[form-round-three]]
- done when: deciding the last item keeps the module; scenario test

changed: no switch after the last item, config form_auto_next removed; header tab gets ✓, note line '✓ all done here · l next module'; l from a done module passes empty ones; scenario test; commits: f32ada4; try: decide the last inbox item
