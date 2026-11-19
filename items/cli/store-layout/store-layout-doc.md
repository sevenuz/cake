---
id: store-layout-doc
tags: [doc]
parent: "[[store-layout]]"
created: 2026-11-19T12:16:28+01:00
by: claude-code
sessions: [59209220-8a77-4046-9145-aa74c0881358]
owner: sevenuz
---
# Store layout design

Context: agreed in chat 2026-11-19.
Decided:
- A store is always a `.cake` dir: `<dir>/.cake`. A git repo gets a worktree on `git_branch` (cake/main) there, a dir without git a plain `.cake`. Lookup stays: nearest `.cake` up the tree, then a linked dir from stores.json, then the global store.
- `--store`, `CAKE_STORE`, a path in `mv --to-store` and the store paths in stores.json may name the dir or its `.cake`: `~/git/work` means `~/git/work/.cake`.
- `cake --store <dir> init` only links the current dir to an existing store (with `--parent <goal>`). Without a store there it fails and says to run `cake init` in that dir. It never creates a store, so it can no longer link a dir like `~` by accident.
- The global store is `~/.local/share/cake/.cake` and is created like any other: `cake -g init` makes `~/.local/share/cake` a git repo (empty first commit on main) and then runs the normal init there. Before that it is a plain `.cake`, created on first use.
- `cake init` adopts a plain `.cake` that already has items (the global store used before init, a project used before `git init`): the items move into the new worktree and are committed.
- Old layout: the global dir or a store listed in stores.json that has `items/` right in it is moved into `<dir>/.cake` on first use (the whole dir, its git repo included), with a note on stderr. Such a `.cake` is its own repo on main, which lookup and sync handle like a worktree.
- The global store is never registered in stores.json under a name; it stays `global`.
Rejected:
- Recognizing the store's own dir in the lookup while keeping both layouts: fixes the symptom, keeps two code paths.
- Moving any dir with an `items/` folder: a code repo may have one; only the global dir and listed stores move.
Edge cases: adopting into an existing remote branch with the same item ids fails and names the dir with the leftover items.
