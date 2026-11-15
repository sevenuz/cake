---
id: tree-detail-box
status: done
parent: "[[tree-form]]"
created: 2026-11-14T12:56:54+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 36, out: 19768, cache_read: 4071920, cache_write: 28108}}
closed: 2026-11-14T20:22:04+01:00
due: 2026-11-20
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:56:54+01:00/2026-11-14T13:01:26+01:00
  - 2026-11-14T13:09:11+01:00/2026-11-14T13:09:11+01:00
---
# Detail box under the tree, focus with tab

- why: text opened in the tree is cut and repeats in the card; O got the user lost; answering needs l to go on
- what: see [[tree-form-design]] (second round): tree on top for navigation, full content of the hovered row in a box below, <tab> focus, e editor, o only hidden children, O removed, sort renamed inbox, review colors, form_auto_next
- done when: scenario tests pass with the new keys, user tried it in the terminal

changed: tree on top (navigation, o = hidden children only), box below with the full hovered row (fields, text, options); <tab>/<cr> focus into the box, <esc>/<tab> back; actions also from the tree; e opens the editor; auto next module after the last open item (config form_auto_next); sort renamed inbox (your config updated); review ◈ in its own color; O removed; commits: 6314e30; try: cake

comment (claude-code, 2026-11-14): user (chat): o also one more parent; color per row kind like review; right columns with titles, colored, dimmer, with deadline — done in 35de9a9

changed: columns with titles (status who prio due deadline; narrow: status who due), muted colors (late red, soon yellow, prio 1 red, agents blue); module items in the module color with status mark, done green, parents dim, o-children dim slanted; o shows one more parent; commits: 6314e30, 35de9a9; try: cake
