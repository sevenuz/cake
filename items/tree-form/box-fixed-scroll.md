---
id: box-fixed-scroll
status: task
parent: "[[tree-form]]"
created: 2026-11-14T21:34:43+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Detail box: fixed height, own scrolling, contained keys

- why: the detail box changes height with every row, and gg in the box jumped to the top of the tree
- decided (user, 2026-11-14): the box takes half the screen, the tree the other half; text scrolls inside on overflow
- keys act on the focused pane only (tab switches): in the tree j/k gg G c-d c-u move the cursor; in the box they scroll the text
- box with options (answer, review), decided: j/k move the option, gg/G first/last option, c-d/c-u scroll the text
- done when: moving through rows never resizes the box; gg/G in the box never move the tree; tests
