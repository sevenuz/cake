---
id: form-modules
status: done
tags: [plan]
parent: "[[workflow-v2]]"
blocked_by: ["[[agent-ready-rules]]"]
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
closed: 2026-11-13T23:50:41+01:00
assignee: claude-code
owner: sevenuz
---
# Form as modules: news, sort, answer, review, mine, triage

- why: the form should follow your day and be startable in parts
- `cake` runs the order from config `form` (default news, sort, answer,
  review, mine, triage); `cake form review` or `cake form sort answer`
  runs only those; `dump` stays as an optional module
- news: what agents did since your last run (new subtasks, started,
  finished, questions), per line ok / hold (back to you) / drop
- answer: tasks assigned to you that block someone, options a-e or own
  words; review: ok / change (appended, back to the assignee) / drop
- mine: your open tasks by score: start, done, drop, later
- triage: runs the `triage` alias if there is one
- done when: each module runs alone and in the default order, tested in
  a 60 column terminal

changed: cake runs config form (news, sort, answer, review, mine, triage), cake form <modules> runs single ones, dump optional, form_width; commit 5aeb0b5; try: cake form review
