---
id: skill-load-hooks
status: task
parent: "[[skill-load]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: ai
owner: sevenuz
---
# Claude Code hooks: load the skill, block .cake edits

- why: a new session did not load the cake skill and an agent edited an item file directly
- decided (user, 2026-11-14): SessionStart hook says "this repo uses cake: load the cake skill first" when a .cake/ exists (fits [[qzf5dz]]); PreToolUse hook denies Edit/Write on paths under .cake/ with "use the cake CLI"
- only for this Claude Code install (~/.claude/settings.json); other agents: [[store-foreign-edits]]
- done when: a fresh session in this repo loads the skill; an Edit on .cake/items is refused with the hint
