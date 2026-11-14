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

Agreed in chat 2026-11-14. Cross-cutting rules; each subtask carries its own details.

- one widget, five filters: news, sort, answer, review, mine show the same tree; the module decides which items match (editable) and which only give context
- context rows (parents, revealed siblings) are dimmed and not editable in this module
- parents: up to `tree_depth` levels above a match (config, default 2); deeper ancestors collapse into one `… ›` row before the oldest shown parent
- hidden siblings: the parent shows `+N` (N children not shown)
- one open key everywhere (`open`: `o <space>`, as today): on any row, also a context row, it shows the body under the line and reveals hidden children (dimmed); again closes both. Rejected: separate keys for body and children (inconsistent with the other views)
- full subtree: `O` opens an overlay with the whole tree from the root above the hovered item; j/k, `o` open; `<cr>` closes and jumps to the item if this module has it; read only (one place to edit); closes with `<esc>` or `q`. `A` stays sort's "decision for the rows below"
- jk (config `keys.escape`) only leaves text input; in navigation j moves at once (answer to jk-in-navigation)
- saving: every change is written at once (undo `u` as today); items changed this run stay in the module with a ✓ mark until the form quits, even when they no longer match the filter (e.g. inbox → task in sort)
- edit line: under the hovered editable entry, `status · who · prio · due · deadline`; no field cursor, the existing keys (t/n/x, w, p, :due, :deadline) set fields, the line only shows them (answer to edit-line-fields)
- sort default who: config `sort_who`, default "ai"; "-" nobody, "me" or a name
- rejected: tables with heading rows (parents blend in, see how-set-parent-headings); editing inside the overlay (two places to edit); whole subtrees as context (too long)

Example (sort, tree_depth 2):

```
… › Form
    Dashboard: one screen            +2
    ▸ Gantt view of tasks             inbox
      task · who:ai · prio – · due – · deadline –
    ✓ Time per goal                   task ai
```
