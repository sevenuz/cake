---
id: option-wrap-indent
status: task
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Wrapped answer options indent under their first word

- why: a long option wraps to column 0 and the a)/b) labels get lost
- continuation lines start where the option's first word starts (after `a) `)
- done when: a long option renders aligned; test with a narrow width
