---
id: tree-form-design
tags: [doc]
parent: "[[tree-form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
owner: sevenuz
---
# Tree form: decisions

Agreed in chat 2026-11-14 (second round after testing). Build from this.

- one layout for all modules (news, inbox, answer, review, mine): the tree on top, a detail box below; no box around the tree
- tree: navigation only; module items bright (answer `? ` in the question color, review its own mark and color), parents and revealed rows dimmed; parents up to `tree_depth` (config, default 2), deeper ones as `… ›`; `+N` hidden children
- `o` on the tree only shows or hides hidden children (no text in the tree any more)
- detail box: always the full content of the hovered row: title, fields line (status · who · prio · due · deadline), whole body, and for answer/review the options; context rows show content but take no actions
- focus: `<tab>` switches between tree and box (also `<cr>` from the tree into the box); in the box j/k choose options (answer, review) or scroll the text, `<esc>`/`<tab>` back
- action keys (t n dd w p c s i digits :) work from the tree too, on the hovered item (answer 1a): sorting stays `t j t j`; in answer/review n/p jump between questions
- `e`: open the hovered item in the editor (`cake edit <id>`), the form pauses and redraws after
- every key saves at once; changed items stay with ✓ until quit; `u` undo
- after the last open item of a module the form goes on to the next module; config `form_auto_next` (default true)
- module `sort` is renamed `inbox`; no old name kept (nobody else uses it yet)
- rejected: `O` full tree overlay (o reveals what is hidden; a whole-tree form may come later), text opened inside the tree (shown twice with the box), actions only in the box (too many keys for sorting)
