---
id: stats-sections
status: done
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-16T02:25:18+01:00
by: claude-code
commits: [b637502, 359c1f5]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-16T17:04:09+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T02:25:19+01:00/2026-11-16T02:29:22+01:00
tokens_in: 9
tokens_out: 6939
tokens_cache_read: 1792276
tokens_cache_write: 9400
---
# Stats overlay: sections rhythm, flow, agents, fun

- why: the stats overlay shows little of the data cake has
- v cycles: overview · rhythm (hour×weekday heatmap, records, chat vs form) · flow (cycle time per goal, inbox age, answer speed, funnel) · agents (estimate accuracy, cost per done item, top token items, parallel share, wait on you) · fun (footprint as km/tea/phone charges, words you vs agents)
- d period as before; tab scope as before
- done when: all sections render on the real store; tests for the numbers

changed: s then v cycles overview → rhythm → flow → agents → fun, j/k scroll, d period, h/l tab; rhythm heatmaps look back 4 weeks; tests for grid/merge/median; try: cake, s, v

rework (user): all sections on one page under the table (two columns from 120 cols), j/k scroll, no v; agent time now per session: timers started together in one session count once, and the agent's chat turns count (logged by the hook from now on)
