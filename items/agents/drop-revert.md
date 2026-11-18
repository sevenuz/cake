---
id: drop-revert
status: done
tags: [discuss]
parent: "[[agents]]"
created: 2026-11-16T02:25:18+01:00
by: claude-code
commits: [80ab744]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T15:35:34+01:00
assignee: sevenuz
owner: sevenuz
time:
  - 2026-11-16T02:25:19+01:00/2026-11-16T02:25:25+01:00
tokens_out: 197
tokens_cache_read: 152948
tokens_cache_write: 727
---
# Dropped items with commits get reverted

- why: drop only closes the ticket; its commits stay in main
- decided (user, 2026-11-16): an agent creates a revert task (assignee the agent, owner the user) for dropped items with commits; cake writes nothing itself
- skill rule in the agent run; tag the dropped item reverted when the revert task exists
- done when: skill rule written

changed: skill step 3: dropped items with commits and without #reverted get a revert task (assignee the agent, owner you) under the same parent, the dropped item gets #reverted; cake itself writes nothing

discuss (sevenuz, 2026-11-16): do they get automatically reverted? explain how it works

answer: not automatic; agent creates a revert task you review
