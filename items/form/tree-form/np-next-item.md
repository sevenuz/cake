---
id: np-next-item
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T17:36:37+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:12:27+01:00/2026-11-15T17:23:59+01:00
tokens_in: 24
tokens_out: 8190
tokens_cache_read: 1569095
tokens_cache_write: 12677
---
# n/p: next item of interest in every tree

- why: n/p walked questions only in answer and review; elsewhere they were note/prio
- what: n/p go to the next/previous open module item everywhere; N note, P prio
- design: [[form-round-three]]
- done when: n/p work in all modules, key line and help show N/P; scenario test updated

changed: n/p (and J/K) jump to the next/previous open item in every module; N note, P prio; inbox A became :all; key line starts with n/p next prev; scenario updated; commits: f32ada4; try: cake form, n/p, N in inbox
