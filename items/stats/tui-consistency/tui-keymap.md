---
id: tui-keymap
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 3h
---
# Dashboard keys through the shared keymap

Context: the dashboard matches raw key codes: no sequences (gg in stats opens gantt because g = gantt), no G, <c-d>/<c-u>; modifiers are ignored (<c-d> changes the period, <c-e> edits, <c-p> opens the pie); the user's `keys` config changes only the form.
Why: movement and rebinding work in the form but not in the dashboard.
Change: run the dashboard through the form's keymap (sequences, modifiers, config keys) with an act-within-timeout for its poll loop; dashboard actions get names in the action table.
Done when: gg/G/<c-d>/<c-u> move in every dashboard list and overlay, <c-d> no longer changes the period, a rebound key in config works in the dashboard; tests cover gg in stats.
