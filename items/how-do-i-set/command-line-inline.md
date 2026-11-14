---
id: command-line-inline
status: review
parent: "[[how-do-i-set]]"
created: 2026-11-14T00:48:19+01:00
by: claude-code
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546, 81954e9f-9185-4d55-a366-5581de2aacd6]
usage: {claude-opus-5-5: {in: 4, out: 2110, cache_read: 1681137, cache_write: 2873}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-14T01:03:32+01:00/2026-11-14T01:04:45+01:00
---
# Command line on one line with a real cursor

- why: ":" opens a prompt line ":" and a second line "▸ _" below; unclear whether to type ":" again; "_" looks like typed text
- one line: `:` then the typed text, like vim; other text inputs the same (prompt, then text on one line)
- terminal cursor at the end of the text (ratatui set_cursor_position) instead of the drawn "_"
- empty command line shows a dim hint: due, deadline, who, prio, move, w, q
- done when: screen dump shows `:prio 2` on one line with the cursor after it; esc and jk still cancel

changed: : and the text on one line, real cursor after it, dim hint of the commands when empty; esc and jk still cancel; commit d75f957; try: cake form sort, then :
