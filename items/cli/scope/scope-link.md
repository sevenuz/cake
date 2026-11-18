---
id: scope-link
status: task
parent: "[[scope]]"
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 2h
---
# Scope: dirs entries with a default goal

Why: the link has to carry the goal before anything can use it.
Change: stores.json `dirs` entries accept `{"path", "parent"}` next to plain strings; `init --parent <id>` writes it after checking the id; `rename` updates it.
Done when: old and new formats load, init and rename tested.
