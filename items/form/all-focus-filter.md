---
id: all-focus-filter
status: done
parent: "[[form]]"
created: 2026-11-15T18:47:52+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-15T19:48:42+01:00
assignee: ai
owner: sevenuz
---
# ALL tab: focus view, f cycles

- why: ALL shows parked ideas and docs next to work; the user wants what needs doing, ranked, but still be able to see everything
- decided (user, 2026-11-15): default focus = open tasks (task, doing, review, inbox) and their parents; children ordered by score, highest first
- f cycles: focus → + notes (parked ideas, docs) → + closed (done, dropped) → focus; the title line says which
- N in ALL makes a task a note (park it right there), t makes it a task again
- done when: ALL starts in focus, f shows the other two, N parks; scenario test

changed: ALL starts with open work (inbox, task, doing, review) and all their parents, in score order; f cycles to + notes and + done/dropped, the title says which; N parks a task as note, t back to task; test + scenario; commits: d6e6755; try: cake form review, l, f f, N on a task
