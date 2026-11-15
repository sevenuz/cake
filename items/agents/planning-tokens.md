---
id: planning-tokens
status: task
parent: "[[agents]]"
created: 2026-11-14T22:49:48+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# Planning tokens counted nowhere

- why: in chat mode the discussion before "go" runs without a timer; the usage hook only gives tokens to items whose timer runs, so planning cost is lost
- options to weigh: a session item that collects untimed answers; give them to the next item started in the session; a per-store "untracked" bucket
- context: [[chat-mode-design]]; the hook is `cake usage claude-hook` (Stop hook, reads the session transcript)
- done when: untimed tokens of a session show up somewhere in cake stats; decided with the user first
