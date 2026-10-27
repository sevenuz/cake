---
id: fq5jbv
status: todo
tags: [git]
created: 2026-10-27T12:30:10+01:00
by: claude-code
priority: 1
---
# Merge driver for items/*.md

Field-wise 3-way merge so the conflicts in [[2wkhyp]] resolve automatically:

- tags, blocked_by: set merge (added on either side kept, removed on either side removed)
- time: union of intervals
- status/closed: a side that changed wins; both changed → closed (done/dropped) wins over open, closed = later
- other scalars (parent, due, priority, unknown keys): a side that changed wins; both → ours
- body: git merge-file 3-way; overlapping edits → conflict markers, driver fails, normal conflict flow

Setup: store .gitattributes `items/*.md merge=cake` (committed), `git config merge.cake.driver "cake merge-driver %O %A %B"` set by cake init (local, also on attach). Clones without the driver (phone apps) fall back to git's text merge.
