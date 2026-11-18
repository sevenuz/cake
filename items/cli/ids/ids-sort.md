---
id: ids-sort
status: done
parent: "[[ids]]"
created: 2026-11-18T16:20:08+01:00
by: claude-code
commits: [94a4ece, bfddad6]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T17:09:36+01:00
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T16:46:28+01:00/2026-11-18T16:47:36+01:00
tokens_in: 26
tokens_out: 5839
tokens_cache_read: 1445363
tokens_cache_write: 18013
---
# Ids: the sort step offers a new id

Why: the user names an item best while sorting it.
Change: the form's sort step shows the id and a key to rename it inline, prefilled with a suggestion from the title (topic words); rename goes through the normal rename so links follow.
Done when: rename while sorting works in the form via tmux, tested.

changed: ':id <new>' in inbox, mine, review and all renames the row via cake rename (children, blockers, links follow), the cursor stays on it, it does not count as sorted; no prefilled suggestion (the title slug is gone, a suggestion would bring it back); try: cake form inbox, :id merge-driver
