---
id: tokens-per-item
status: done
tags: [stats]
parent: "[[cli]]"
created: 2026-11-13T20:43:48+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-13T23:07:14+01:00
---
# Tokens per item with an energy, CO2e and water estimate

- why (user): see what the AI work on a todo costs, next to its time
- items get `usage`: in, out, cache read, cache write tokens per model
- Claude Code: Stop hook `cake usage claude-hook` (in ~/.claude/settings.json)
  reads only new transcript lines, gives each answer to the items of that
  session whose timer ran; running items collect it until they stop
- any agent: `cake usage add <id> --model M --in N --out N ...`
- estimate in `cake show` and `--json` (footprint), all numbers in config
  `footprint`; rough, deliberately on the high side
- commit: 87ac0f4
- try: work with Claude on an item with a running timer, stop it, then
  `cake show <id>`: a "tokens ... Wh ... CO2e ... water" line
