---
id: fields-migrate
status: review
parent: "[[fields]]"
blocked_by: ["[[fields-stats]]", "[[fields-hook]]"]
created: 2026-11-18T16:46:24+01:00
by: claude-code
commits: [20174b9]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h
---
# Fields: old usage and sessions keys in existing items

Why: old items carry `usage` per model and `sessions`; stats should still count them.
Change: decide in the work: map old `usage` to tokens_in/tokens_out once via migrate.rs, or leave and declare; ask if it touches the user's items.
Done when: this store's stats match before and after.

## Log
- 2026-11-18 changed: cake migrate --only usage|workflow; applied --only usage on this store: 70 items' usage per model became tokens_in/out/cache_read/cache_write (header only); habitats_tcc had none; the old tag workflow step was not run (it would have moved 2 items here and 9 there); try: cake stats -p month
