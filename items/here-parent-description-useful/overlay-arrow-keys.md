---
id: overlay-arrow-keys
status: task
parent: "[[here-parent-description-useful]]"
blocked_by: ["[[overlay-wrap-cursor]]"]
created: 2026-11-14T11:18:40+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2]
assignee: ai
owner: sevenuz
---
# Feedback box: move the cursor with arrow keys

- why: in the feedback box you can only type and delete at the end; you cannot go back to fix a word
- left/right move one character, home/end to line start/end, up/down one wrapped line
- typing and backspace act at the cursor; delete removes the character after it
- same editing for the one-line input in the card (`:` commands), sharing one small text buffer
- done when: a typo in the middle can be fixed without deleting everything after it; tests for insert, delete and moves
