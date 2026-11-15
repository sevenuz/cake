---
id: av6hve
tags: [data]
parent: "[[agents]]"
created: 2026-10-27T13:11:05+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Training data: export tasks with their agent sessions

- link item → agent sessions via `sessions`, transcripts from the agent (Claude Code: ~/.claude/projects/*/<session>.jsonl)
- example = idea + plan + tasks + transcripts + commits + outcome (review → done/dropped, time)
- export command, e.g. `cake export --sessions` as JSONL
- privacy: opt-in per store, strip secrets, never in published branches
- open: format, which outcomes count as good
