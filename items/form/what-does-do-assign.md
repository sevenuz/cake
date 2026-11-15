---
id: what-does-do-assign
status: review
tags: [form-feedback]
parent: "[[form]]"
blocked_by: ["[[news-who-fix]]"]
created: 2026-11-14T00:24:46+01:00
by: sevenuz
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546, eb1412a2-56ac-4478-9717-6ebac99000ac]
due: 2026-11-15
deadline: 2026-11-16
assignee: ai
owner: sevenuz
---
# what does do=assign do here? it happend when i pressed w, makes no sense, no?

- where: news table, row form-keys (cake)

comment (sevenuz, 2026-11-14): answer: yes, it makes no sense as shown.
- w in the news table cycles the assignee (you, ai, agents, people); the row's "do" column then says "assign"
- the news table has no "who" column, so you see "assign" but not to whom
- on save (enter, or h/l to the next module) it runs `cake assign <id> <who>`; a started item goes back to task
- fix options in [[news-who-fix]] (question below)
