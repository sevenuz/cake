---
id: fq5jbv
status: todo
tags: [git]
created: 2026-10-27T12:30:10+01:00
by: claude-code
priority: 1
---
# Merge driver for items/*.md

Goal: frontmatter conflicts from [[2wkhyp]] resolve automatically on sync.

Setup
- store `.gitattributes`: `items/*.md merge=cake` (committed)
- `cake init` (also attach) sets local `merge.cake.driver = cake merge-driver %O %A %B`
- clones without the driver (phone apps): git text merge as today

Driver `cake merge-driver BASE OURS THEIRS`
- parse all 3 leniently as items, write result to OURS via the normal serializer
- exit 0 clean, 1 if body has conflict markers

Field rules (3-way: unchanged side takes the other; equal → keep)
- `tags`, `blocked_by`: set merge, adds of both kept, removes of both applied
- `time`: union by start; same start, one stopped → stopped one
- `status`+`closed` together: closed (done/dropped) wins over open; both closed → later `closed`; closed result stops open intervals at `closed`
- `parent`, `due`, `priority`, unknown keys, both changed differently: ours wins
- `by`: never changes
- body: `git merge-file` 3-way; overlap → markers, exit 1

Sync
- rm vs change (git skips the driver): keep the changed item, `git add` it, continue the merge
- external edits (Obsidian etc.) are not auto-committed before sync

Tests
- play-through scenarios as integration tests, all clean except overlapping body edits
- unit tests per field rule
