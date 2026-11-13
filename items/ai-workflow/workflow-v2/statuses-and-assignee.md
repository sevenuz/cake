---
id: statuses-and-assignee
status: done
tags: [plan]
parent: "[[workflow-v2]]"
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
usage: {claude-opus-5-5: {in: 98, out: 43840, cache_read: 30590236, cache_write: 66781}}
closed: 2026-11-13T23:50:19+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-13T23:16:48+01:00/2026-11-13T23:25:40+01:00
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

changed: statuses inbox/note/task/doing/review, assignee and owner, commands note/review/assign/comment, cake migrate; commit a5b04ea; try: cake add -m x, then sort it in cake
