---
id: wf-depth
parent: "[[workflow-doc]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Tree depth: by meaning, no number

Context: "at most 3 levels" was only a skill line (cake walks 10 levels, MAX_DEPTH) meant to stop endless agent nesting; it forced docs and handoffs to odd places.
Decided (user, 2026-11-18, option 1a):
- No depth number.
- A child belongs to exactly one parent and is part of it: the parent is finished only when the child is.
- Anything that touches several parents gets a `[[link]]` or `--blocked-by`, not a place in the tree.
- Tasks split as deep as needed; a task with work children is a plan, only leaves are worked.
- Goals nest: `goal` is a tag, so `phd` › `gekkofs` › `gekkofs-paper` works; rules and goals are read up the tree.
- Docs go under their plan (option 2a); a `#doc` note's children are untagged notes, one topic each, linking each other with `[[id]]`; the parent body is a short summary, no index (the tree shows the children).
Rejected: a higher fixed limit like 5 (still arbitrary); subtasks never split again (unergonomic, user 2026-11-18).
