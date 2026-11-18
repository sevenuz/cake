---
id: scope-doc
tags: [doc]
parent: "[[scope]]"
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Scope design

Context: agreed in chat 2026-11-18 (answers 1a, 2b, 3a). A linked dir can name a default goal; captures land under it and views show only its subtree.
Decided:
- Where: in stores.json next to the link, per machine like the link itself (1a).
  ```json
  "phd": { "store": "~/git/phd/.cake",
           "dirs": [ {"path": "~/git/gekkofs", "parent": "gekkofs"},
                     {"path": "~/papers"} ] }
  ```
  A plain path string in `dirs` stays valid (no default goal).
- Set with `cake --store <store> init --parent <id>` in the dir; the id must exist in that store.
- Applies from the linked dir and every dir below; the deepest link wins; a `.cake` in the tree always wins.
- `add` and form capture without `-p` land under the goal; `-p -` puts an item at the top level; `-g` and `--store` ignore the default. Output names where it landed (`→ gekkofs`).
- Views scoped (2b): `ls`, `ready` and the form show only the goal's subtree; `--all` shows the whole store; `-p` still picks another subtree. Agents in that dir work only on it.
- `stats` stays store-wide.
- Form sort module (3a): an inbox item with a preset parent shows as `gekkofs ▸ <item>`; the sort keys keep the parent, `p` moves it.
- `rename` of the goal updates stores.json; a missing goal → top level with a warning, never an error.
Rejected: default on the goal note as a `dirs:` field (synced, survives renames, but puts one machine's paths into the store).
Edge cases: goal dropped or done → still the default until unlinked, warning in `status`; two stores linking the same dir → deepest wins, equal depth → error at init.
