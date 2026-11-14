---
id: overlay-wrap-cursor
status: task
parent: "[[here-parent-description-useful]]"
created: 2026-11-14T11:18:40+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2]
assignee: ai
owner: sevenuz
---
# Feedback box: grow and place cursor by word wrap

- why: typing feedback past the line end, the new line stays hidden and the cursor sits in the wrong place, so the user types blind
- cause: the box height and cursor position are computed from the character count divided by the width, but the text is drawn with word wrap, which breaks lines earlier
- compute the wrapped lines the same way the text is drawn (word wrap at the inner width) and take box height and cursor row/column from them
- done when: the box grows as soon as a word moves to the next line, cursor always right after the last typed character; test with a long word and many short words
