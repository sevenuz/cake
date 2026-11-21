---
id: stats-cake-labels
status: review
parent: "[[stats]]"
created: 2026-11-18T23:37:14+01:00
by: sevenuz
commits: [d426284]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
time:
  - 2026-11-21T02:02:11+01:00/2026-11-21T02:03:51+01:00
---
in the stats we have that field, cake baked and slices eaten. how is that defined and is it hard coded? i dont know if we need that. no one will understand it currently anyway

## Log
- 2026-11-20 comment (claude-code): plan (triage, user chose a): the labels 'baking time' and 'slices eaten' (hard-coded in the stats summary, src/insights.rs) become 'time tracked' (your time plus agents) and 'done'; done when: cake stats and the dashboard show the plain labels, tests updated.
- 2026-11-21 changed: the stats section 'the cake' is now 'total': 'time tracked' (yours and agents', say so) and 'done' (N items) instead of 'baking time' and 'slices eaten'. try: cake stats, or the stats view of the dashboard
