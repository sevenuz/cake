---
id: form-nothing-waiting
status: task
parent: "[[form]]"
created: 2026-11-19T23:21:42+01:00
by: sevenuz
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
---
after opening the phd store and reviewing items i got:

jul@julspc:~/Documents/phd/gkfs_hybrid(main)$ cake
review 3
nothing waiting
nothing waiting
nothing waiting


three times "nothing waiting", where does it come from?


also the dashboard tells me 3 items in inbox but when i open it, its over 40.

## Log
- 2026-11-20 comment (claude-code): plan (triage): cake prints 'nothing waiting' once per empty module (three empty modules: three lines); print one summary line instead. The dashboard inbox count (3) and the form inbox (40+) count different scopes; find which and make both use the same. Done when: one line for empty modules and equal inbox counts, both tested.
