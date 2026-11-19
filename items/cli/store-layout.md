---
id: store-layout
status: task
parent: "[[cli]]"
created: 2026-11-19T12:16:28+01:00
by: claude-code
sessions: [59209220-8a77-4046-9145-aa74c0881358]
assignee: claude-code
owner: sevenuz
estimate: 3h
---
# Stores: one layout, every store in a .cake dir

Context: there were two kinds of store. A project store is `<repo>/.cake` (a worktree on cake/main); a separate store made with `cake --store DIR init`, and the global store, kept `items/` right in their dir. Lookup only knows the first kind by walking up, so inside a separate store's own dir cake silently used the global store.
Why: two layouts, two code paths, and a silent fallback that puts items into the wrong store.
Change: as in [[store-layout-doc]].
Done when: `cake --store ~/git/work init` links to `~/git/work/.cake`, `--store` on a dir without a store fails, the global store lives in `~/.local/share/cake/.cake`, old layouts move on first use, tests pass.
