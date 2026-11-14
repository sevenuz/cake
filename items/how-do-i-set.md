---
id: how-do-i-set
status: done
tags: [form-feedback]
created: 2026-11-14T00:40:04+01:00
by: sevenuz
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546]
closed: 2026-11-14T01:34:02+01:00
assignee: ai
owner: sevenuz
---
# how do I set !? and if i press : i get another input under : and i dont know if i have to type : again or not. its not but its unintuitinve, why do i have a _ as marker for input?

- where: sort table, row what-does-do-assign (cake)

comment (sevenuz, 2026-11-14): (from claude-code) answer:
- "!" is the priority column: p cycles 1, 2, 3, none; or `:prio 2`
- after ":" the first line is the prompt, the second your text: type `prio 2`, no second ":"
- "_" is a fake cursor drawn after the text
- fixes split into [[prio-column-header]] and [[command-line-inline]]
