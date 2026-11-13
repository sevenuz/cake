---
id: how-should-sync-ask
status: todo
tags: [ask]
parent: "[[sync-passphrase]]"
created: 2026-11-13T18:23:28+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# How should sync ask the passphrase only once?

- a) ssh-agent with AddKeysToAgent in ~/.ssh/config, no code (recommended)
- b) cake sets SSH multiplexing for its git calls (small code change)
- c) leave it
