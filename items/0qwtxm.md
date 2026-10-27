---
id: 0qwtxm
status: todo
tags: [cli]
created: 2026-10-27T12:30:10+01:00
by: claude-code
priority: 2
---
# cake edit detects concurrent changes

cake edit opens the editor on a copy; if the item file changed meanwhile (Claude or another shell), merge with the merge driver logic or refuse instead of overwriting. Same working copy lost-update case from [[2wkhyp]].
