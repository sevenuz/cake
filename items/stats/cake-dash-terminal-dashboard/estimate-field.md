---
id: estimate-field
status: doing
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T22:37:30+01:00/
---
# Estimate field for task durations

- why: the critical path to a goal's deadline needs durations ([[dash-design]])
- cake add/edit --est 2h, shown in show and json; skill: agents estimate when they split
- fallback: median tracked time of done tasks of the same kind (agent or user)
- done when: field round-trips; fallback tested