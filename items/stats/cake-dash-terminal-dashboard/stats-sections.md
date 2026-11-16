---
id: stats-sections
status: review
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-16T02:25:18+01:00
by: claude-code
commits: [b637502]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 9, out: 6939, cache_read: 1792276, cache_write: 9400}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T02:25:19+01:00/2026-11-16T02:29:22+01:00
---
# Stats overlay: sections rhythm, flow, agents, fun

- why: the stats overlay shows little of the data cake has
- v cycles: overview · rhythm (hour×weekday heatmap, records, chat vs form) · flow (cycle time per goal, inbox age, answer speed, funnel) · agents (estimate accuracy, cost per done item, top token items, parallel share, wait on you) · fun (footprint as km/tea/phone charges, words you vs agents)
- d period as before; tab scope as before
- done when: all sections render on the real store; tests for the numbers

changed: s then v cycles overview → rhythm → flow → agents → fun, j/k scroll, d period, h/l tab; rhythm heatmaps look back 4 weeks; tests for grid/merge/median; try: cake, s, v
