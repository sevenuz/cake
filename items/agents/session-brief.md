---
id: session-brief
status: review
tags: [workflow]
parent: "[[agents]]"
blocked_by: ["[[skill-load-hooks]]"]
created: 2026-10-27T13:29:58+01:00
by: claude-code
commits: [7fc1e4c]
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
priority: 1
time:
  - 2026-11-21T01:57:29+01:00/2026-11-21T02:01:48+01:00
---
# Session start hook and brief

- Claude Code SessionStart hook: `cake -g sync`, then running, review, ask, ready top 3, ideas
- skill: open every session with a 5-line brief
- offer moving project-tagged phone ideas
- done when: fresh session starts with the brief

split (2026-11-15): the hook that loads the skill is [[skill-load-hooks]]; this item is only the 5-line brief at session start, after it

## Log
- 2026-11-21 changed: the SessionStart hook (claude-code-guard.py) adds a brief of the store: doing, questions for you, waiting for review, ready top 3, inbox count, at most 5 lines; the skill says show it as is in the first answer. Not done: cake -g sync at session start (network in a 10 s hook, and the skill says no sync unless asked) and offering to move project ideas from the phone. try: start a new claude session in this repo
