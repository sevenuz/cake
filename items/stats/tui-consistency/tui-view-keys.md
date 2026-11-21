---
id: tui-view-keys
status: task
parent: "[[tui-consistency]]"
blocked_by: ["[[tui-keymap]]"]
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 2h
---
# Dashboard views on digits, same letters as the form

Context: the same letter means different things: s stats vs timer, p pie vs previous, d period vs dd drop, g gantt vs gg top, <tab> store tab vs focus, <esc> quit vs back; in the form news s is "stop agent", mine s the timer.
Why: muscle memory from the form breaks in the dashboard and back.
Change (proposal, confirm first): dashboard views on 1-4 (main, pie, stats, gantt), period on [ and ], v stays measure/view; then s/p/d/g mean what they mean in the form; news "stop agent" gets its own key or name.
Done when: no letter means two different things across dashboard and form; help and footers show the new keys.
