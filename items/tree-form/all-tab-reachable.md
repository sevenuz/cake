---
id: all-tab-reachable
status: review
parent: "[[tree-form]]"
created: 2026-11-15T17:12:06+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:15:14+01:00/2026-11-15T17:23:59+01:00
---
# ALL tab reachable when nothing waits

- why: with nothing waiting `cake form` prints a line and exits, so ALL cannot be opened
- what: open the form anyway, see [[nothing-waits-dump]]; l reaches all
- design: [[form-round-three]]
- done when: empty-waiting store opens the form, l reaches all; scenario test

changed: with nothing waiting the form opens anyway (dump first, see nothing-waits-dump), esc goes on to all; scenario test; commits: f32ada4; try: cake in a store with nothing waiting, esc
