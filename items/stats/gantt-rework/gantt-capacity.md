---
id: gantt-capacity
tags: [idea]
parent: "[[gantt-rework]]"
created: 2026-11-20T21:56:15+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
owner: sevenuz
---
# Gantt capacity: working hours per day

Context: the reworked gantt runs one assignee's tasks one after another in clock time, so 30h of work look like a bit more than one day.
Idea: a config value for the user's working hours per day (e.g. 6h, weekdays only); the schedule only places the user's tasks inside those hours, so the path stretches over real days. Agents keep clock time.
Not decided: parked by the user on 2026-11-20 in favor of plain ordering, see [[gantt-doc]].
