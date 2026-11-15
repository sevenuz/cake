---
id: undo-answer-text
status: task
parent: "[[form-polish]]"
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
# Undo of an answer leaves its text

- why: u on an answered question sets it back to task, but the appended "answer: …" line stays in the body; the agent may read it as answered
- what: undo of an answer also removes the line it appended (or appends "answer withdrawn"); same for a review comment
- done when: answer, u, the body is as before or clearly says withdrawn; scenario test
