---
id: overlay-arrow-keys
status: task
parent: "[[here-parent-description-useful]]"
blocked_by: ["[[overlay-wrap-cursor]]"]
created: 2026-11-14T11:18:40+01:00
by: claude-code
sessions: [4fa59fcd-dfb0-420a-8037-705b420d66a2, cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Feedback box: up/down by wrapped line

- why: in a long feedback text that wraps over several lines, the cursor can only go left/right; reaching a word two lines up takes many presses
- already done by [[input-arrow-keys]]: left/right, ctrl-left/right per word, home/end, del, ctrl-w in every input
- left: up/down move the cursor one wrapped line in the feedback box, keeping the column where possible
- done when: up/down in a three-line feedback text land on the line above/below; test
