---
id: triage-alias
tags: [plan]
parent: "[[workflow-v2]]"
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Shell aliases and the triage prompt

- why: the AI run should start from `cake` without cake knowing any AI
- aliases starting with `!` run a shell command (like git), e.g. in the
  config template:
  `triage: "!claude -p --max-turns 40 --allowedTools 'Bash(cake *)' 'Run the agent run of the cake skill for this store: apply answers and comments, split and work tasks assigned to ai until nothing is ready or reviews are full, then a short summary.'"`
- --max-turns is the emergency stop against runaway; max_review is the
  normal brake; code work needs more tools than `cake *`, listed in the
  template comment
- the triage module and `cake triage` run it; without the alias the
  module is skipped
- done when: `cake triage` runs a real agent pass on a scratch store
