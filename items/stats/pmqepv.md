---
id: pmqepv
status: review
tags: [stats]
parent: "[[stats]]"
blocked_by: ["[[35xst1]]", "[[3s7aa0]]"]
created: 2026-10-27T13:29:58+01:00
by: claude-code
commits: [41b28c6]
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
priority: 1
time:
  - 2026-11-18T22:34:52+01:00/2026-11-18T22:37:11+01:00
tokens_in: 16
tokens_out: 4358
tokens_cache_read: 2884553
tokens_cache_write: 7624
---
# Today, week and streak in cake status

- uses the stats module
- done when: shown in prompt and status line, tested

## Log
- 2026-11-18 changed: cake status segments week (you + agents this week, from timers and the session log) and streak (days in a row, shown from 2); new default list mine, others, ready, today, week, streak, review; your config.hjson has its own status list, add week and streak there; ~18 ms with them on this store; try: cake status
