---
id: store-foreign-edits
parent: "[[skill-load]]"
created: 2026-11-14T22:57:08+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: ai
owner: sevenuz
---
# cake notices item files changed outside cake

- why: hooks only guard Claude Code; any agent or editor can change item files directly, unseen and uncommitted
- every cake command checks the store for uncommitted changes in items/ first; if found: validate the files, commit them as "outside edit" and say so in one line (stderr; in --json a `warning` field)
- broken frontmatter: refuse with the file name instead of committing
- done when: a hand-edited item is reported and committed on the next cake call; tests
