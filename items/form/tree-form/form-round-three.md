---
id: form-round-three
tags: [doc]
parent: "[[tree-form]]"
created: 2026-11-15T17:11:57+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
owner: sevenuz
---
# Form round 3: one layout, o, legend, n/p, new items

Agreed in chat 2026-11-15 after testing the tree form.

- one layout for every screen of the form (tree modules, dump, empty module, end screen): tree on top, details box at the bottom, both fixed halves so nothing jumps while moving or typing
- the box loses its field line (`inbox · who – · prio – · due – · deadline –`); set fields show only after the title in the tree row ([[tree-rows-design]]); the box keeps title, "what" (news), "this run" and the body
- `o` on a row opens one parent up, all its children (the row's siblings, each with `+N` when it has hidden children) and the row's own hidden children; `o` again closes
  - rejected: parent and siblings only (own children only reachable from one level up)
- legend of the status marks (◇ inbox, ○ task, ◐ doing, ◈ review, ✓ done, ✗ dropped, plain note, `+N` hidden children, `… ›` parents cut) in the `?` overlay
- `n`/`p` in every tree: next/previous open item of the module (what still needs a decision); `N` note, `P` prio take over the old n/p; `J`/`K` stay
- `a`: overlay to add a new item at any step, goes to the store's inbox top level; `A`: same, as child of the hovered row; the old `A` (inbox: this row's decision for the rows below) becomes `:all`
- no automatic switch after the last item of a module (`form_auto_next` removed): the module stays, its header tab gets a ✓ and the note line says "all done here · l next module"
  - why: the switch is not noticeable and confuses; rejected: keep auto switch with a note
- ALL must be reachable: `cake` / `cake form` with nothing waiting opens the form anyway ([[nothing-waits-dump]]), the header lists all modules and `all`, l walks to it
- the user tests with the `cake` on PATH (`~/.local/bin/cake` → target/release/cake): every change ends with `cargo build --release`
