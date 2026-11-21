---
id: stats-handoff
tags: [handoff]
parent: "[[stats]]"
created: 2026-11-21T03:21:33+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
---
# Stats and TUI: handoff

State (2026-11-22): the code review round is merged and released (3d5fab8), see [[review-round-summary]]; no open branches or worktrees. Decided for [[tui-consistency]]: dashboard views on digits 1-4, period on < and >, v stays measure/view ([[tui-view-keys]]); filter/search with the full syntax ([[tui-filter-design]]).
Next step: after the reviews are checked (the brake holds agent work while 8+ wait), start [[tui-keymap]] (now easier: dashboard keys live in src/dash/, the form's key table in src/form/keys.rs), then tui-view-keys and [[tui-filter]]. commands.rs is 1148 lines and could be split next.
