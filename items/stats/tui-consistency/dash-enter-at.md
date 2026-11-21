---
id: dash-enter-at
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 1h
---
# Dashboard <cr> opens the form on the selected item

Context: <cr> on a dashboard row opens a form module without --at, so the cursor lands elsewhere; `at` is honored only by `form all`; questions (tasks of yours blocking others) show under "your focus" with module mine, but mine excludes them.
Change: pass --at <id> for item rows, honor `at` in every module; the own list uses the form's rule (questions go to answer) and routes those rows to answer.
Done when: <cr> on any item row opens the right module with the cursor on that item; test.
