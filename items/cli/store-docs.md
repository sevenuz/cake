---
id: store-docs
status: review
parent: "[[cli]]"
created: 2026-11-19T12:16:28+01:00
by: claude-code
commits: [ca7c763]
sessions: [59209220-8a77-4046-9145-aa74c0881358]
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-19T12:24:14+01:00/2026-11-19T13:34:28+01:00
tokens_in: 11
tokens_out: 5144
tokens_cache_read: 554070
tokens_cache_write: 130082
---
# Docs: how stores work and how to set them up

Context: the user wants a `docs/` dir in the repo with short paragraphs and command examples.
Why: the store setup (project stores, a work store used from several repos with goals, the global store) is only explained in the skill and in chat.
Change: `docs/stores.md`: how cake finds a store, the three setups with commands, another machine, sync. Example uses `work`, not phd. Shown to the user as a short sample first.
Done when: the user approved the sample and the file is committed.

## Log
- 2026-11-19 changed: docs/stores.md with lookup order, project store, a work store used from several repos with goals, linking, global store, another machine, sync, old layouts; example is work; try: read docs/stores.md
