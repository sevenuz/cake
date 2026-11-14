---
id: tree-answer
status: done
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[tree-sticky]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
usage: {claude-opus-5-5: {in: 0, out: 210, cache_read: 68242, cache_write: 419}}
closed: 2026-11-14T20:30:11+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:32:04+01:00/2026-11-14T12:32:04+01:00
---
# Answer module on the tree

- why: questions are shown in a tree but only questions can be focused; the user wants to walk parents too
- what: starts on the first question; `n`/`p` next/previous question, j/k every row, open (`o <space>`) body and children; `<cr>` on a question enters answer mode (options 1-9, i text); `<esc>`/jk leave answer mode back to the tree
- answered questions stay with ✓ (tree-sticky)
- done when: scenario test answers two questions with n between them and walks a parent with j/k

changed: answer on the tree: starts on the first question, n/p and J/K jump questions, j/k every row, o opens; <cr> card under the tree (j/k or 1-9, i own words, <esc> back); digits answer straight from the tree; answered questions stay with ✓ and the answer; commits: 0b0a56d; try: cake form answer
