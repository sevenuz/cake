---
id: triage-garden
status: review
parent: "[[ai-workflow]]"
created: 2026-11-15T18:10:11+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T18:10:11+01:00/2026-11-15T18:10:11+01:00
---
# Garden step in triage and agent runs

- why: triage only planned downwards; loose top-level items, duplicates and vague old tasks piled up (user, 2026-11-15: workflow feels out of control)
- what: skill step 4 garden: file top-level items under goals (parent only), ask merge questions for overlaps, ask plan-or-park for vague ready tasks, propose <topic>-polish for leftover buckets; ids always expressive (topic first, goals one word)
- done when: next cake triage files and asks instead of only planning

changed: skill/SKILL.md step 4 garden + Names: always an expressive id; triage prompt example in config doc mentions gardening; one-time cleanup done by hand (form goal, form-polish, items filed under exqdqc/zcgwhr, dashboard merged under tjmbph, 6 October ideas parked as notes); commits: 97007dd; try: cake triage
