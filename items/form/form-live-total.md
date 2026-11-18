---
id: form-live-total
status: task
parent: "[[form]]"
created: 2026-11-18T17:12:47+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 45m
---
# Form: live total in the module header

Context: the header counts a module's items once when it opens; parents that go to review by themselves when their last child closes arrive later, so the review header showed 10/8.
Why: the count should stay true while the run changes what waits.
Change: the total grows when items arrive during the run (handled / handled + still waiting), in every module (decided, user 2026-11-18, option a).
Done when: closing the last child in review shows its parent and the header reads 9/10, not 9/8; tested via tmux.
Not: newcomers as `+2` (cluttered); closing plans without review (their Done when needs a check).
