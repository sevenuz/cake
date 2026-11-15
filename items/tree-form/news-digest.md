---
id: news-digest
status: task
parent: "[[tree-form]]"
created: 2026-11-15T17:47:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# News as a read-only digest

- why: news looked like a to-do list (count in the progress bar, "finished" read like a status, w gave "do=assign"); it is only what agents did since the last form run
- decided (user, 2026-11-15): read-only digest
  - shows agent items new or started since the last run; items in review are left out (REVIEW has them)
  - its count is in the header but not in the progress bar; no ✓ / "all done"; key line says "read only"
  - actions: s stop (the agent's item back to you as a task, was w hold), dd drop; no w, no : fields
  - l goes on and passes empty modules; with only news waiting the form starts at news, not dump
  - rejected: remove the module (marks in ALL instead); keep it with renamed labels
- done when: scenario test (s gives it back to you); tmux check
- also closes [[news-drop-w]]
