---
id: statuses-and-assignee
status: todo
tags: [plan]
parent: "[[workflow-v2]]"
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Statuses inbox/note/task/review, assignee and owner

- why: one status field says where an item is; tags stay plain labels
- `cake add` creates `inbox`; statuses inbox, note, task, doing, review,
  done, dropped; `cake do` sets task
- fields `assignee` and `owner`; `me` and `ai` are input shorthands,
  stored are real names (you: config `user`, else git user.name)
- `cake migrate` shows what changes, then applies it: old notes → note,
  ask → task for you blocking its parent, reply → appended to the
  parent's text, review tag → status review, unapproved AI plans →
  inbox, todo/doing get an assignee from `by`; tags triaged/sorted go
- done when: migrate on a copy of this store shows a sensible overview
  and the tests cover every status change
