---
id: dash-overlays
status: done
parent: "[[cake-dash-terminal-dashboard]]"
blocked_by: ["[[dash-tabs]]"]
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-16T00:03:08+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:32:45+01:00/2026-11-15T23:36:15+01:00
tokens_in: 12
tokens_out: 9123
tokens_cache_read: 1784284
tokens_cache_write: 16483
---
# Dashboard: pie and stats overlays

- p pie: m time/tokens/done, d period, <cr> zoom store → goals, <bs> back
- s stats: table per store (you, agents, tokens, kWh, CO2e, done, sparkline), total row
- done when: both render from the stats core; keys tested

changed: p pie overlay (braille, ~10 rows, legend with value and %, you/agents, Wh/CO2e/water), m time/tokens/done, d period, j/k slice, <cr> zooms store → goals, <bs> back; s stats overlay (you, agents, tokens, kWh, CO2e, done, sparkline per store, all row, overdue/review/questions/streak); q/esc back; refreshes every 2 s; commits: 6bdc5b7; try: cake, p, <cr>, m, d, s
