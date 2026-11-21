---
id: tui-filter
status: task
parent: "[[tui-consistency]]"
blocked_by: ["[[tui-keymap]]"]
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 24h
---
# Filter and search in dashboard and form

Context: design in [[tui-filter-design]] (proposal, to confirm with the user first). Today :filter is dropped silently when a module has no match and header counts ignore it.
Change: implement the confirmed design: one query with / everywhere, kept across modules and into the form, tokens, header chip and counts, 0-match state.
Done when: the design's example works end to end; scenario tests for /, <esc>, h/l with 0 matches.
