---
id: agents-md
status: doing
parent: "[[agents]]"
created: 2026-11-18T17:14:29+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 30m
time:
  - 2026-11-18T17:14:29+01:00/
---
# Dev rules: AGENTS.md instead of the skill

Context: rules for developing cake (git history, tests, live binary) went into the cake skill, which is also used on other machines and for other stores; some lived only in one machine's agent memory.
Why: the skill should only say how to use cake; rules for working on cake belong to the repo.
Change: AGENTS.md at the repo root (CLAUDE.md imports it) with the dev rules; the skill keeps a generic git line; dev rules leave the agent memory.
Done when: the skill has no cake-dev rule and AGENTS.md has them all.
