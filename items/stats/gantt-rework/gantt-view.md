---
id: gantt-view
status: task
parent: "[[gantt-rework]]"
blocked_by: ["[[gantt-lanes]]"]
created: 2026-11-20T21:56:29+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 2h30m
---
# Gantt view: all targets in one view

Context: the dashboard's critical path view (paint_path in src/dash.rs) shows one goal per page with one deadline line, see [[gantt-doc]].
Change: one block per target on a shared axis labeled in days; targets further than 4 weeks written as an arrow at the right edge; deadline solid, due dashed, colored by slack (green, yellow within 3 days, red late); nested targets as markers; the block's critical path in the mine color; header with date and slack.
Done when: a text dump of the view on a fixture with two targets and a nested deadline shows both blocks, the marker and the slack, checked in a throwaway store.