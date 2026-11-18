---
id: review-reopen
status: done
parent: "[[form]]"
created: 2026-11-18T22:00:00+01:00
by: claude-code
commits: [091c223, 9e988cb]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T23:33:08+01:00
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-18T22:32:04+01:00/2026-11-18T22:33:35+01:00
tokens_in: 18
tokens_out: 5354
tokens_cache_read: 3150964
tokens_cache_write: 7882
---
# Review: change an ok into not ok yet

Context: in the form's review module, an ok closes the item and its row takes no further decision; the user sometimes notices a problem right after.
Why: a wrong ok should be fixable on the spot, without leaving the form.
Change: on a row decided ok in this run, the not ok option (and i) still works: the item goes back to task with #discuss and the doubts, as from review; undo (u) keeps working too.
Done when: ok, then not ok on the same row gives a task with #discuss and the doubts line; tested via tmux.

## Log
- 2026-11-18 changed: in the review module a row you ok'd or dropped in this run keeps its options; not ok yet sends it back as a task with #discuss and your doubts, like from review; tmux: ok on Parent, k, 3, 'too vague' gave task #discuss with the log line; try: cake form review, 1 on a row, k back, 3
