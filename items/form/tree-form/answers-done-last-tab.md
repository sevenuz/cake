---
id: answers-done-last-tab
status: done
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 16, out: 3859, cache_read: 1026552, cache_write: 6338}}
closed: 2026-11-14T21:27:19+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:58:00+01:00/2026-11-14T20:59:57+01:00
---
# Jump to the last tab after the last answer

- why: after answering every question the user has to switch tabs by hand
- when no open question is left in answer, the form switches to the last tab
- done when: answering the last question shows the last tab; test

changed: cause found: items changed in one module (answered questions) also showed in all later modules of the run, so after the last answer the form stopped in review with your answers; now changed items stay only in the module where you changed them; after the last answer the form goes on to the next module with items, or the end screen; scenario test; commits: 145bd63; try: answer all questions, the form moves on by itself. Ask: did 'last tab' mean the end screen even when review/mine have items?
