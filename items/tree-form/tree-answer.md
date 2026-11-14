---
id: tree-answer
status: task
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[tree-sticky]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
---
# Answer module on the tree

- why: questions are shown in a tree but only questions can be focused; the user wants to walk parents too
- what: starts on the first question; `n`/`p` next/previous question, j/k every row, `o` fold; `<cr>` on a question enters answer mode (options 1-9, i text); `<esc>`/jk leave answer mode back to the tree
- answered questions stay with ✓ (tree-sticky)
- done when: scenario test answers two questions with n between them and walks a parent with j/k
