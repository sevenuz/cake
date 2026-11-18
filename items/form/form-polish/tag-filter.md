---
id: tag-filter
status: done
parent: "[[form-polish]]"
created: 2026-11-15T22:56:23+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T23:17:24+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:07:04+01:00/2026-11-15T23:10:21+01:00
tokens_in: 16
tokens_out: 4197
tokens_cache_read: 1862855
tokens_cache_write: 8655
---
# Tree: filter by tag

- why: user wants to narrow ALL (and the modules) to one tag
- e.g. :tag doc (or a tag step in the f cycle); the title says the filter; :tag clears
- done when: ALL shows only rows with the tag plus their parents

changed: :tag NAME in any module (also #NAME) keeps only items with that tag plus their parents, notes included even in ALL's open-work view; the title shows #NAME in the tag color; :tag clears; a tag with no match turns the filter off with a note; the filter stays across modules until cleared; commits: 016bfdb; try: cake, ALL, :tag goal
