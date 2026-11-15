---
id: no-auto-next
status: task
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# Module stays after its last item, with a done mark

- why: the automatic switch to the next module is not noticeable and confuses
- what: no switch, config form_auto_next removed; header tab ✓, note "all done here · l next module"
- design: [[form-round-three]]
- done when: deciding the last item keeps the module; scenario test