---
id: deadline-field-asks-score
status: done
tags: [plan, review]
parent: "[[ai-workflow]]"
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
closed: 2026-11-13T18:07:29+01:00
---
# Deadline field, asks and score in cake

- `deadline` (hard) next to `due` (soft), same parsing/clearing
- answer = `cake done <ask> -m TEXT` (appends, closes, unblocks); no
  extra `answer` command; `drop -m` appends a reason
- score = deadline pressure (hard ≫ soft) + 10 per item waiting on it
  (transitively) + prio 30/20/10 + age ≤ 20 days; ls/ready sort by it
- score in `--json`
- done when: tests for deadline, answer, score order ✓
- commit: b7846db
