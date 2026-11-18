---
id: triage-one-line
status: task
parent: "[[triage-output]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: ai
owner: sevenuz
---
# Triage: one line out, usage on a triage note

- why: `cake triage` prints the agent's whole output, which is already in the store; its cost is nowhere
- decided (user, 2026-11-14): print one line: questions and tasks created, tokens, time; store the usage on one note per store tagged `triage` (created on first run), so stats show it
- how: the alias runs `claude -p … --output-format json`; cake reads the result (usage, duration, last summary line) when the alias output is JSON
- items created: count by agent and time window of the run
- done when: `cake triage` prints one line; the triage note's usage grows per run
