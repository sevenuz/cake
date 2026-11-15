---
id: cake-morning-terminal-form
status: done
tags: [plan]
parent: "[[ai-workflow]]"
blocked_by: ["[[deadline-field-asks-score]]"]
created: 2026-11-13T17:35:30+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-13T20:13:26+01:00
---
# cake morning: terminal form

- `cake` alone (terminal) or `cake morning`
- 1 paper: one line = one todo by user, empty line ends
- 2 questions: open asks by score, letter picks option line, text = answer
- 3 planned: agents' notes with `done when`, no open ask: do · do with
  due/deadline · drop · skip · quit
- summary: added, answered, approved, ready for agents / for you
- each change is its own cake call: no lock while waiting
- termimad Question, no new dependency
- also: `cake do` (approve), space separated ids
- done when: full pass on a scratch store ✓, on the real store (user)
- commit: f45844f
- try: run `cake`; your note "the input is always at the bottom" below is solved by the TUI (3cc0626)
