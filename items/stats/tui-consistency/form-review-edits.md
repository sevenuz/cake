---
id: form-review-edits
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 45m
---
# Review: field edits are no decision

Context: review blocks w/P but allows :who and :prio; `:prio 2` marks the row decided (✓) and moves the progress bar.
Why: a prio change is not an ok or a drop.
Change: allow field edits (who, prio, due, tag, est) in review consistently, and do not count them as decisions in review, answer and mine (as rename and parent already are not).
Done when: :prio in review leaves the item undecided; w/P work in review; test.
