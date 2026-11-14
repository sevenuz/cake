---
id: news-drop-w
status: task
parent: "[[what-does-do-assign]]"
created: 2026-11-14T11:18:26+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2]
assignee: ai
owner: sevenuz
---
# Remove w from the news table

- why: w in the news table changes the assignee, but the table has no who column, so the row only says "assign" without saying to whom (user answer c in [[news-who-fix]])
- remove the who key (w) from the news table: key handling, the key line at the bottom and the help list
- reassigning stays possible in the mine table
- drop or adapt the form test that presses w in the news table
- done when: w in the news table does nothing, key line and help no longer list it, tests pass
