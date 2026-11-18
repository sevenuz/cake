---
id: scope-sort
status: done
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
commits: [42336d2]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
usage: {claude-opus-5-5: {in: 4, out: 1562, cache_read: 444020, cache_write: 2616}}
closed: 2026-11-18T21:57:43+01:00
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-18T17:20:51+01:00/2026-11-18T17:22:34+01:00
---
# Scope: sort module shows the preset parent

Why: a preset parent must be visible while sorting, or wrong guesses slip through.
Change: inbox items with a parent show as `<parent> ▸ <item>`; sort keys keep it, `p` moves it.
Done when: tested in the form via tmux.

## Log
- 2026-11-18 changed: the inbox already shows a preset parent as the row above the item (tree), so only moving was missing: ':parent <id>' (and '- ' for the top level) in inbox, mine, review and all; not counted as sorted; 'p' stays previous; checked in tmux; try: cake form inbox, :parent <goal>
