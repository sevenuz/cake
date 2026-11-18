---
id: dash-round2
status: done
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-16T00:29:54+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-16T02:25:18+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T00:29:54+01:00/2026-11-16T00:32:05+01:00
tokens_in: 8
tokens_out: 3860
tokens_cache_read: 1493844
tokens_cache_write: 9194
---
# Dashboard: item gantt, header time and user

- why: user feedback 2026-11-16: the true per-item gantt was missing, header should show time and user
- gantt v cycles who → stores → items → path; items: one row per item worked in the window, timer bars plus the user's chat time on it (light)
- header: HH:MM and the user name
- stats: 'per hour/day/month' instead of 'per bucket'
- done when: four gantt views render; tests

changed: gantt v cycles who → stores → items → path; items: a row per item worked in the window, timers █ and your chat on it ▒; header top border 'user · Mon 00:31'; stats column says per hour/day/month; commits: be700dc; try: cake, g v v
