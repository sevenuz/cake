---
id: nothing-waits-dump
status: task
parent: "[[if-nothing-todo-then]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Nothing waits: open dump with a hint

- why: `cake` with nothing waiting prints one line and exits; the user is left without a next step
- decided (user, 2026-11-14): open the form at the dump screen; above it a hint: "N tasks ready for agents: tell Claude 'go' in a session, or run `cake triage` / `cake work`"
- hint only lists aliases that exist in the config; no ready tasks: no hint line
- done when: `cake` in a store with nothing waiting opens dump with the hint; scenario test
