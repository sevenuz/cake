---
id: scope-sort
status: task
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h
---
# Scope: sort module shows the preset parent

Why: a preset parent must be visible while sorting, or wrong guesses slip through.
Change: inbox items with a parent show as `<parent> ▸ <item>`; sort keys keep it, `p` moves it.
Done when: tested in the form via tmux.
