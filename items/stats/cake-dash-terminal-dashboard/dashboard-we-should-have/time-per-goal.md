---
id: time-per-goal
status: review
parent: "[[dashboard-we-should-have]]"
blocked_by: ["[[session-log]]"]
created: 2026-11-14T11:18:51+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T23:14:54+01:00/2026-11-15T23:14:54+01:00
---
# Time per goal for a period

- why: the dashboard's pie needs tracked time summed per goal for the last day, week, month or year
- sum each item's timer entries into its top-level goal (note tagged goal up the tree; items without one go to "other")
- clip entries to the period (an entry crossing the start counts only its part inside); a running timer counts up to now
- available as data for the dashboard, no new dependency
- done when: tests for clipping, running timer, items without goal

changed: per-goal numbers in the stats core (nearest note tagged goal, rest 'other'), clipped to the period, running timers until now; tests for clipping, running timer, items without goal; commits: c55bacb; try: cake stats -p week
