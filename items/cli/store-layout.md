---
id: store-layout
status: review
parent: "[[cli]]"
created: 2026-11-19T12:16:28+01:00
by: claude-code
commits: [35dadc2]
sessions: [59209220-8a77-4046-9145-aa74c0881358]
assignee: claude-code
owner: sevenuz
estimate: 3h
time:
  - 2026-11-19T12:16:28+01:00/2026-11-19T12:24:14+01:00
tokens_in: 53
tokens_out: 21746
tokens_cache_read: 3505785
tokens_cache_write: 34678
---
# Stores: one layout, every store in a .cake dir

Context: there were two kinds of store. A project store is `<repo>/.cake` (a worktree on cake/main); a separate store made with `cake --store DIR init`, and the global store, kept `items/` right in their dir. Lookup only knows the first kind by walking up, so inside a separate store's own dir cake silently used the global store.
Why: two layouts, two code paths, and a silent fallback that puts items into the wrong store.
Change: as in [[store-layout-doc]].
Done when: `cake --store ~/git/work init` links to `~/git/work/.cake`, `--store` on a dir without a store fails, the global store lives in `~/.local/share/cake/.cake`, old layouts move on first use, tests pass.

## Log
- 2026-11-19 changed: every store is a .cake dir; --store DIR (or DIR/.cake) init only links to an existing store and fails otherwise; global store moved to ~/.local/share/cake/.cake and created by the normal init (-g init makes the dir a git repo first); init adopts a plain .cake used before git; old-layout global and listed stores move into <dir>/.cake on first use; skill intro updated; try: cake --store ~/git/work init in a linked dir, cake -g init
