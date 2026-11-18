---
id: ids
status: task
parent: "[[cli]]"
created: 2026-11-18T16:20:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
---
# Ids: random on capture, named while sorting

Context: ids were a slug of the first 4 title words; for dumped sentences that gave `we-want-make-cake` or `when-i-started-new`.
Why: ids should be short topic words; a sentence slug is neither short nor a topic.
Change (user, 2026-11-18):
- `cake add` without an id gets a random id (`kqejfn`); the slug and its stop word filter go away.
- The form's sort step offers a better id, prefilled, the user confirms or edits (option b).
- Triage renames random or sentence ids to topic ids (topic first, 1–3 words), listed in the summary (option a, skill in [[wf-skill]]).
Not: a better slug with more filler words (still weak for sentences).
Done when: the children are done.
