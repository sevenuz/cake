---
id: input-arrow-keys
status: review
parent: "[[tree-form]]"
created: 2026-11-14T20:51:28+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
usage: {claude-opus-5-5: {in: 4, out: 1544, cache_read: 319185, cache_write: 4677}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T21:15:29+01:00/2026-11-14T21:16:35+01:00
---
# Arrow keys and ctrl-arrows in every input field

- why: input fields only type and delete at the end
- left/right move a character, ctrl-left/right jump a word, home/end, delete
- every input: `:` line, feedback box, edit fields; one shared text buffer
- extends [[overlay-arrow-keys]]
- done when: a typo mid-line can be fixed everywhere; tests for word jumps

changed: one shared text buffer with a cursor for the : line, the feedback box, own words and change requests, the dump line; left/right a char, ctrl-left/right a word, home/end and ctrl-a/ctrl-e, del, ctrl-w deletes the word before; typing and backspace at the cursor; the help overlay lists them; not done: up/down per wrapped line in the feedback box (from [[overlay-arrow-keys]]); test buf_edits_at_the_cursor_and_jumps_words, checked in tmux; commits: 178da59; try: F, type, ctrl-left, fix a word
