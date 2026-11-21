---
id: form-silent-keys
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 2h
---
# Form: a note for unsupported keys, help per module

Context: on editable rows keys a module does not support do nothing (t/f/N in mine, c/s in inbox, dd/c/w/P in answer, w/P in review, s in all), while context rows get a note; help lists every key in every module (mine shows t, N, f, 1-9).
Why: silence looks like a bug; help promises keys that do nothing.
Change: one table of supported actions per module; unsupported keys show "<key> <action>: not in <module> (try :… / all)"; help is built from the same table and dims the rest.
Done when: every unsupported key gives a note; help per module matches what works; unit test over the table.
