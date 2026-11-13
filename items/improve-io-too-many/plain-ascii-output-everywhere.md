---
id: plain-ascii-output-everywhere
status: todo
tags: [cli, review]
parent: "[[improve-io-too-many]]"
created: 2026-11-13T19:05:26+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Plain ASCII output everywhere

- why (user): too many non-ASCII signs and ugly ~~~~ dividers
- done: the morning form is plain (7348abf)
- left: `cake ls` markers (· ○ ◐ ▶ ✓ ✗) and tree lines (├─ └─),
  `cake show` (· separators, ~~~~ for a markdown rule from the skin)
- option: ASCII by default (o todo, > doing, x done, - dropped, tree with
  |- and \`-), unicode behind a config switch
- done when: ls, show, status and the form print ASCII only by default
