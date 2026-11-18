---
id: form-live-total
status: review
parent: "[[form]]"
created: 2026-11-18T17:12:47+01:00
by: claude-code
commits: [faaa898]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 45m
time:
  - 2026-11-18T22:31:17+01:00/2026-11-18T22:32:04+01:00
tokens_in: 10
tokens_out: 3188
tokens_cache_read: 1716966
tokens_cache_write: 6723
---
# Form: live total in the module header

Context: the header counts a module's items once when it opens; parents that go to review by themselves when their last child closes arrive later, so the review header showed 10/8.
Why: the count should stay true while the run changes what waits.
Change: the total grows when items arrive during the run (handled / handled + still waiting), in every module (decided, user 2026-11-18, option a).
Done when: closing the last child in review shows its parent and the header reads 9/10, not 9/8; tested via tmux.
Not: newcomers as `+2` (cluttered); closing plans without review (their Done when needs a check).

## Log
- 2026-11-18 changed: when a module shows more items than it counted (a parent goes to review after its last child), the module count and the total grow; tmux: ok on the child, header REVIEW 2 and 1 / 2; try: cake form review on a plan's last child
