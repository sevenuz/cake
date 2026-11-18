---
id: ls-l-tree
status: task
parent: "[[ls-long]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: ai
owner: sevenuz
---
# ls -l: plain tree with bodies under their rows

- why: `cake ls -l` prints each item as a separate block with a long meta line and a rule between; the tree is lost and it reads badly
- decided (user, 2026-11-14): stays non-interactive (agents and pipes use it); browsing is `cake form all`
- each row as in `cake ls` (tree lines, mark, id, title), its body rendered and indented under it at the tree depth, one dim line of fields (status · who · prio · due) only when set
- no rule lines between items; a blank line between top-level items
- `--json` unchanged
- done when: `cake ls -l -p <plan>` reads as one tree; test on plain output
