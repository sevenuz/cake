---
id: review-reopen
status: doing
parent: "[[form]]"
created: 2026-11-18T22:00:00+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-18T22:32:04+01:00/
---
# Review: change an ok into not ok yet

Context: in the form's review module, an ok closes the item and its row takes no further decision; the user sometimes notices a problem right after.
Why: a wrong ok should be fixable on the spot, without leaving the form.
Change: on a row decided ok in this run, the not ok option (and i) still works: the item goes back to task with #discuss and the doubts, as from review; undo (u) keeps working too.
Done when: ok, then not ok on the same row gives a task with #discuss and the doubts line; tested via tmux.
