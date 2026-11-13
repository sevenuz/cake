---
id: deadline-field-asks-score
status: todo
tags: [plan]
parent: "[[ai-workflow]]"
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Deadline field, asks and score in cake

- `deadline` (hard) next to `due` (soft), same parsing/clearing
- `cake answer <id> -m TEXT`: appends answer to ask body, sets done
  (unblocks the asking item)
- score = deadline pressure (hard ≫ soft) + leverage (items blocked on
  it, transitively) + prio + inbox age; `ls/ready --sort score`
- score in `--json`
- done when: tests for deadline, answer, score order

