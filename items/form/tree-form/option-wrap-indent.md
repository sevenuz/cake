---
id: option-wrap-indent
status: done
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-14T21:28:41+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:57:29+01:00/2026-11-14T20:57:34+01:00
tokens_in: 2
tokens_out: 330
tokens_cache_read: 121860
tokens_cache_write: 1486
---
# Wrapped answer options indent under their first word

- why: a long option wraps to column 0 and the a)/b) labels get lost
- continuation lines start where the option's first word starts (after `a) `)
- done when: a long option renders aligned; test with a narrow width

changed: answer and review options wrap word by word; continuation lines start under the option's first word, not at column 0; test wrapped_options_hang_under_their_first_word; commits: e599a48; try: a question with a long option in a narrow pane
