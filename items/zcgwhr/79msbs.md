---
id: 79msbs
status: done
tags: [cli]
parent: "[[zcgwhr]]"
blocked_by: ["[[store-list]]"]
created: 2026-10-27T13:29:58+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6]
closed: 2026-11-13T20:11:01+01:00
priority: 2
---
# cake mv --to-store

- move an item (and children) between stores, e.g. -g idea → project
- keeps id, by, created, sessions
- target: name from stores.json, `global`, or a path; top-level there
- refuses ids that exist in the target; target locked + committed on its own
- done when: tested in both directions ✓
- commit: 45cf693
- try: `cake -g add -m "test idea"`, then `cake -g mv test-idea --to-store cake`: it appears in this store, gone from -g
