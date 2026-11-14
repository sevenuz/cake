---
id: input-arrow-keys
status: doing
parent: "[[tree-form]]"
created: 2026-11-14T20:51:28+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T21:15:29+01:00/
---
# Arrow keys and ctrl-arrows in every input field

- why: input fields only type and delete at the end
- left/right move a character, ctrl-left/right jump a word, home/end, delete
- every input: `:` line, feedback box, edit fields; one shared text buffer
- extends [[overlay-arrow-keys]]
- done when: a typo mid-line can be fixed everywhere; tests for word jumps
