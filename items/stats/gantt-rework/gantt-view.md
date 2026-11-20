---
id: gantt-view
status: review
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-lanes]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
commits: [b9d0f00]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 2h30m
time:
  - 2026-11-20T22:18:52+01:00/2026-11-20T22:22:04+01:00
---
# Gantt view: all targets in one view

Context: the dashboard's critical path view (paint_path in src/dash.rs) shows one goal per page with one deadline line, see [[gantt-doc]].
Change: one block per target on a shared axis labeled in days; targets further than 4 weeks written as an arrow at the right edge; deadline solid, due dashed, colored by slack (green, yellow within 3 days, red late); nested targets as markers; the block's critical path in the mine color; header with date and slack.
Done when: a text dump of the view on a fixture with two targets and a nested deadline shows both blocks, the marker and the slack, checked in a throwaway store.

## Log
- 2026-11-20 changed: the critical path view (g, v three times) shows every target of the store in one view: a block per target with deadline, due, days left and spare or late time (green > 3 d, yellow, red); deadline lines solid, due lines dashed, inner targets (e.g. submit-register) as lines in the outer block, targets beyond 4 weeks as 'later:' in the header; plans as ▔ spans, reviews ▒, questions ◆; j/k scroll, the axis stays. Checked on a copy of the phd store in a throwaway HOME. try: cake in ~/git/phd after cargo build --release, g, v v v
