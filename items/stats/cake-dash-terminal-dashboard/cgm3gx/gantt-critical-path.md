---
id: gantt-critical-path
status: review
parent: "[[cgm3gx]]"
blocked_by: ["[[gantt-stores]]", "[[estimate-field]]"]
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 10, out: 8501, cache_read: 1681438, cache_write: 9971}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:54:12+01:00/2026-11-15T23:56:56+01:00
---
# Gantt G3: items and critical path to a deadline

- classic gantt with blocked_by; pick a goal with a deadline → forward view now ┊ … │ deadline
- open tasks chained by blocked_by with estimates, questions as ◆; path line fits ✓ / N h over (red)
- the focus list warns when a goal's path misses its deadline
- done when: path computed and tested on a fixture

changed: g then v v: per goal with a deadline (j/k), open tasks from now to the deadline after their blockers, est or ~history/1h, questions ◆, critical path in the mine color with 'fits ✓' or 'N over' (red); the focus list warns per goal that misses; G3 has no past-items view (the who view covers the past); commits: 55dafa6; try: give a goal a deadline (cake edit stats --deadline fri), cake, g v v
