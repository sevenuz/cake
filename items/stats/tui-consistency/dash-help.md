---
id: dash-help
status: task
parent: "[[tui-consistency]]"
blocked_by: ["[[tui-keymap]]"]
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 2h
---
# Dashboard help overlay on ?

Context: ? does nothing in the dashboard; footers are the only help and omit <tab>, <esc>, h/l in overlays and <bs>; the pie footer lists "back" twice.
Change: a ? overlay per dashboard view, generated from the action table (after tui-keymap); footers from the same table.
Done when: ? shows the keys of the current view and they match what works.
