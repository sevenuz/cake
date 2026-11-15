---
id: no-auto-next
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 6, out: 4391, cache_read: 967456, cache_write: 7173}}
closed: 2026-11-15T17:33:26+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:12:59+01:00/2026-11-15T17:23:59+01:00
---
# Module stays after its last item, with a done mark

- why: the automatic switch to the next module is not noticeable and confuses
- what: no switch, config form_auto_next removed; header tab ✓, note "all done here · l next module"
- design: [[form-round-three]]
- done when: deciding the last item keeps the module; scenario test

changed: no switch after the last item, config form_auto_next removed; header tab gets ✓, note line '✓ all done here · l next module'; l from a done module passes empty ones; scenario test; commits: f32ada4; try: decide the last inbox item
