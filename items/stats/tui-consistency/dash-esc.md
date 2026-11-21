---
id: dash-esc
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 15m
---
# Dashboard <esc> does not quit

Context: <esc> on the dashboard main screen quits cake; everywhere else it means back, so one extra <esc> from an overlay quits.
Change: <esc> on the main screen clears a filter (see [[tui-filter-design]]) or does nothing; only q quits.
Done when: <esc> never quits; test.
