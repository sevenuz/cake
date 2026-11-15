---
id: review-order-fixed
status: done
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 6, out: 934, cache_read: 303745, cache_write: 3641}}
closed: 2026-11-14T21:28:46+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:52:00+01:00/2026-11-14T20:52:13+01:00
---
# Review rows keep their order after accepting

- why: accepting a row in review moves it; the user loses their place
- rows keep the order they had when the module opened; a decided row stays where it was
- sorting later only on explicit request (own key), never as a side effect
- done when: accept/drop in review leaves every row in place; scenario test

changed: the form remembers where each item was first seen per store and sorts rows by that, so accept/drop/edits never move a row; new items go last; test rows_keep_the_first_seen_order; commits: 3e0f37e; try: cake form review, accept a row: it stays in place
