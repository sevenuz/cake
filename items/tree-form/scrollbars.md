---
id: scrollbars
status: review
parent: "[[tree-form]]"
created: 2026-11-15T17:36:37+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:37:45+01:00/2026-11-15T17:39:58+01:00
---
# Scrollbars on the tree and the details box

- why: long trees and long texts scroll without showing where you are
- what: a scrollbar on the right of the tree and of the details box (and help overlay) when their content is longer than the room
- done when: scrollbar only on overflow, thumb follows the cursor/scroll; render test; checked in tmux

changed: scrollbar at the right of the tree and on the box's right border, only when the content is longer than the room; thumb of the focused pane in the hint color; help overlay not yet (it does not scroll, see help-per-module); test; commits: 1fb0799; try: cake, all tab, tab into a long text
