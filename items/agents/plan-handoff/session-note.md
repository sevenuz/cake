---
id: session-note
status: dropped
parent: "[[plan-handoff]]"
blocked_by: ["[[agent-metadata-field]]"]
created: 2026-11-14T22:57:08+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T16:15:08+01:00
assignee: ai
owner: sevenuz
---
# Session end: short note with open threads only

- why: handoffs repeated what the store already holds; the next session needs only what no item covers
- decided (user, 2026-11-14): at session end the agent writes a note tagged `session`: only open threads, half-made decisions and context not in any item; no list of touched items (found later with `cake ls --session <id>`, [[agent-metadata-field]])
- skill: length rule becomes "as short as possible while complete"; the session note rule in Chat mode
- done when: skill says both; the next session starts from the newest session note plus `--session`

superseded by [[wf-handoff]]: one living handoff per goal instead of a note per session (user, 2026-11-18)
