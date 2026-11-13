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

- why (user): too many special characters, ugly ~~~~ dividers; both
  modes wanted for every command
- one table in src/style.rs: status markers, tree lines, separators,
  markdown bullet and rule, each with a unicode and an ascii form
- config `style`: auto (ascii without UTF-8 locale or TERM=dumb),
  unicode, ascii; env CAKE_STYLE wins
- color off with NO_COLOR or when output is not a terminal
- done when: ls, show, status and the form follow it ✓ (ce124c2)
- try: `CAKE_STYLE=ascii cake ls`, `NO_COLOR=1 cake show ai-workflow`
