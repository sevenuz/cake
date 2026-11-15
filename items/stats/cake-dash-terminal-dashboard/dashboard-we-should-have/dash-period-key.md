---
id: dash-period-key
status: done
parent: "[[dashboard-we-should-have]]"
blocked_by: ["[[time-per-goal]]", "[[dash-overlays]]"]
created: 2026-11-14T11:18:51+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-16T00:03:08+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T23:36:15+01:00/2026-11-15T23:36:15+01:00
---
# Dashboard: key to switch day, week, month, year

- why: user wants the last-day section switchable to week, month and year
- one key cycles day → week → month → year → day; the section title shows the current period
- key: p (period); user said "maybe f", but f already shows hint keys in the form; confirm in review
- the chosen period is not saved, each start begins with day
- done when: switching redraws the section with the new period's numbers

changed: d in the pie and stats overlays cycles day → week → month → year (title shows it; starts at day each time); key d instead of f/p because p opens the pie; commits: 6bdc5b7
