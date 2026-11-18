---
id: all-tab-reachable
status: done
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T17:36:37+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:15:14+01:00/2026-11-15T17:23:59+01:00
tokens_out: 1436
tokens_cache_read: 356217
tokens_cache_write: 2945
---
# ALL tab reachable when nothing waits

- why: with nothing waiting `cake form` prints a line and exits, so ALL cannot be opened
- what: open the form anyway, see [[nothing-waits-dump]]; l reaches all
- design: [[form-round-three]]
- done when: empty-waiting store opens the form, l reaches all; scenario test

changed: with nothing waiting the form opens anyway (dump first, see nothing-waits-dump), esc goes on to all; scenario test; commits: f32ada4; try: cake in a store with nothing waiting, esc
