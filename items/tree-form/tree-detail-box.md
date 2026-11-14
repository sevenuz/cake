---
id: tree-detail-box
status: review
parent: "[[tree-form]]"
created: 2026-11-14T12:56:54+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:56:54+01:00/2026-11-14T13:01:26+01:00
---
# Detail box under the tree, focus with tab

- why: text opened in the tree is cut and repeats in the card; O got the user lost; answering needs l to go on
- what: see [[tree-form-design]] (second round): tree on top for navigation, full content of the hovered row in a box below, <tab> focus, e editor, o only hidden children, O removed, sort renamed inbox, review colors, form_auto_next
- done when: scenario tests pass with the new keys, user tried it in the terminal

changed: tree on top (navigation, o = hidden children only), box below with the full hovered row (fields, text, options); <tab>/<cr> focus into the box, <esc>/<tab> back; actions also from the tree; e opens the editor; auto next module after the last open item (config form_auto_next); sort renamed inbox (your config updated); review ◈ in its own color; O removed; commits: 6314e30; try: cake
