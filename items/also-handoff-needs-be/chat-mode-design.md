---
id: chat-mode-design
tags: [doc]
parent: "[[also-handoff-needs-be]]"
created: 2026-11-14T21:11:23+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
owner: sevenuz
---
# Chat mode: decide in chat, cake records

- why: discussing in chat, then answering the same thing again in cake, then starting the agent by hand costs every decision twice
- decided (user, 2026-11-14): when the user is in the chat, the chat is the work loop and cake is the record
- questions: asked in chat while the user is here; the agent writes the answer into cake itself (into the task, or a question item created already done with `answer: …`); never asks the same thing again in cake
- go: "go" / "go ahead" in chat starts the work at once in the same session; no separate trigger, no extra agent run
- review: works both ways; "ok <id>" in chat lets the agent run `cake done <id>`; the form stays for later
- cake question items only when the user is away: agent runs, triage, or "decide later"
- planning tokens: chat before "go" has no running timer and is counted nowhere; known gap
- rejected: questions only in cake (slow, double answers); questions only in chat (lost when the user is away)
- written into the skill as section "Chat mode"
