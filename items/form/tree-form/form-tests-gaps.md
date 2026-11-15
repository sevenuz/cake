---
id: form-tests-gaps
status: task
parent: "[[tree-form]]"
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# Form tests: editor key, tmux check, clippy

- why: e (editor) has no test; the sticky display check is skipped without tmux; clippy warns in src/form (contains instead of iter().any, manual checked division, clamp)
- what: scenario for e with EDITOR set to a script that changes the item; note when tmux is missing; fix the three clippy warnings
- done when: cargo test covers e, cargo clippy has no warnings in src/form
