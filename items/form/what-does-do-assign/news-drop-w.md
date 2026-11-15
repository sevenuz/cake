---
id: news-drop-w
status: done
parent: "[[what-does-do-assign]]"
created: 2026-11-14T11:18:26+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 8, out: 1810, cache_read: 764323, cache_write: 3821}}
closed: 2026-11-15T19:48:42+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:47:55+01:00/2026-11-15T17:49:43+01:00
---
# Remove w from the news table

- why: w in the news table changes the assignee, but the table has no who column, so the row only says "assign" without saying to whom (user answer c in [[news-who-fix]])
- remove the who key (w) from the news table: key handling, the key line at the bottom and the help list
- reassigning stays possible in the mine table
- drop or adapt the form test that presses w in the news table
- done when: w in the news table does nothing, key line and help no longer list it, tests pass

changed: w is gone from news (it cycled the assignee, shown as do=assign); stopping the agent is s now, see news-digest; commits: 25c0c4f
