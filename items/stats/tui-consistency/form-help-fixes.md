---
id: form-help-fixes
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 2h
---
# Form help: fit, notation, true texts

Context: help is cut on 40-row terminals with no scroll; keys show as "gg home", "G end", "<c-d> pagedown", "tab details" (Home/End/PageUp/Tab fall into a debug format); the config template lists `feedback "F"` (rejected as unknown) and omits filter f, center zz, tags #; the : help lists only some commands; "q … <c-c> always" is false in text input (<c-c> is escape there); "<c-d> half a page" is 10 rows in the tree, 5 lines in the box; closing help swallows the key in modules but not on the "still" screen.
Change: two columns or j/k scroll in help; vim notation for all keys; template key list and : help generated from the action and command tables; correct texts (or make <c-c> quit in text input and half page relative to the pane); closing help drops the key everywhere.
Done when: help fits at 100x30 and 130x40, notation is uniform, template keys all parse; tests.
