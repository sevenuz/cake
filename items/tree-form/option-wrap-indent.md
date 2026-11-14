---
id: option-wrap-indent
status: review
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
usage: {claude-opus-5-5: {in: 2, out: 330, cache_read: 121860, cache_write: 1486}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:57:29+01:00/2026-11-14T20:57:34+01:00
---
# Wrapped answer options indent under their first word

- why: a long option wraps to column 0 and the a)/b) labels get lost
- continuation lines start where the option's first word starts (after `a) `)
- done when: a long option renders aligned; test with a narrow width

changed: answer and review options wrap word by word; continuation lines start under the option's first word, not at column 0; test wrapped_options_hang_under_their_first_word; commits: e599a48; try: a question with a long option in a narrow pane
