---
id: ids-random
status: task
parent: "[[ids]]"
created: 2026-11-18T16:20:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 30m
---
# Ids: random when none is given

Why: sentence slugs are long and not topics.
Change: `add` without an id uses a random id; `slug_id` and the stop words are removed with their tests.
Done when: tests pass and `cake add -m "we want x"` gives a 6-letter id.
