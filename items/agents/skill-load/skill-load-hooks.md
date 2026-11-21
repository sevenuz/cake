---
id: skill-load-hooks
status: done
parent: "[[skill-load]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
commits: [82398fc]
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-21T01:12:31+01:00
assignee: ai
owner: sevenuz
---
# Claude Code hooks: load the skill, block .cake edits

- why: a new session did not load the cake skill and an agent edited an item file directly
- decided (user, 2026-11-14): SessionStart hook says "this repo uses cake: load the cake skill first" when a .cake/ exists (fits [[session-brief]]); PreToolUse hook denies Edit/Write on paths under .cake/ with "use the cake CLI"
- only for this Claude Code install (~/.claude/settings.json); other agents: [[store-foreign-edits]]
- done when: a fresh session in this repo loads the skill; an Edit on .cake/items is refused with the hint

## Log
- 2026-11-20 changed: skill/hooks/claude-code-guard.py, wired in ~/.claude/settings.json (SessionStart, and PreToolUse on Edit|Write|NotebookEdit): a session in a dir with a .cake store gets 'load the cake skill first'; Write/Edit of files inside any .cake/ is refused with 'use the cake CLI' (checked live: a Write into a test .cake dir was refused). Not caught: shell commands writing into .cake (that is store-foreign-edits). try: start a new claude session in this repo; ask it to edit .cake/items/cli.md
