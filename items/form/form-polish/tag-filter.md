---
id: tag-filter
status: review
parent: "[[form-polish]]"
created: 2026-11-15T22:56:23+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:07:04+01:00/2026-11-15T23:10:21+01:00
---
# Tree: filter by tag

- why: user wants to narrow ALL (and the modules) to one tag
- e.g. :tag doc (or a tag step in the f cycle); the title says the filter; :tag clears
- done when: ALL shows only rows with the tag plus their parents

changed: :tag NAME in any module (also #NAME) keeps only items with that tag plus their parents, notes included even in ALL's open-work view; the title shows #NAME in the tag color; :tag clears; a tag with no match turns the filter off with a note; the filter stays across modules until cleared; commits: 016bfdb; try: cake, ALL, :tag goal
