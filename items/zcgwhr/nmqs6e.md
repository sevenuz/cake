---
id: nmqs6e
status: todo
tags: [search]
parent: "[[zcgwhr]]"
blocked_by: ["[[s1k16e]]"]
created: 2026-10-27T12:19:28+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2]
---
# SQLite index for fast search in big stores

.cake/index.sqlite (gitignored), rebuilt incrementally from mtimes, FTS5 over title/body/tags. Only when file scanning gets slow.

Measured 2026-10-27 (release, 1.3 KB items, no git):
- 1k items: show/done 20 ms, ls --json 37 ms
- 10k items: show/done 178 ms, ls --json 890 ms
- cost is linear: every command parses all files for children/blocks/ready/selectors
- first step when needed: cache of parsed frontmatter keyed by mtime+size (gitignored), re-parse only changed files, bodies read lazily
