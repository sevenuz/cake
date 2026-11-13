---
id: i-lost-my-input
status: task
tags: [form-feedback]
created: 2026-11-14T00:41:13+01:00
by: sevenuz
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546]
assignee: ai
owner: sevenuz
---
# I lost my input in here after moving to review and back. maybe its saved but its definetly not printed

- where: sort table, row jk-not-working-close (cake)

comment (sevenuz, 2026-11-14): (from claude-code) likely cause:
- h/l saves the table, then the sort table reloads from the store
- the sort table only lists inbox items and tasks without assignee; a saved row (note, drop, task for you) no longer matches, so it vanishes
- the decision is saved, but nothing shows what was saved
- question [[lost-input-which]] before a fix
