---
id: box-fixed-scroll
status: done
parent: "[[tree-form]]"
created: 2026-11-14T21:34:43+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 16, out: 4566, cache_read: 1411524, cache_write: 8087}}
closed: 2026-11-15T16:48:59+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T22:41:35+01:00/2026-11-14T22:43:08+01:00
---
# Detail box: fixed height, own scrolling, contained keys

- why: the detail box changes height with every row, and gg in the box jumped to the top of the tree
- decided (user, 2026-11-14): the box takes half the screen, the tree the other half; text scrolls inside on overflow
- keys act on the focused pane only (tab switches): in the tree j/k gg G c-d c-u move the cursor; in the box they scroll the text
- box with options (answer, review), decided: j/k move the option, gg/G first/last option, c-d/c-u scroll the text
- done when: moving through rows never resizes the box; gg/G in the box never move the tree; tests

changed: tree and details box each take half the screen, the box never resizes while moving; its text scrolls inside; with focus in the box (tab) j/k, c-d/c-u, gg/G scroll the box text, never the tree; with options (answer, review) j/k and gg/G move the option, c-d/c-u scroll the text; tests box_height_is_fixed, checked in tmux; commits: d18d005; try: cake form, tab, G then gg
