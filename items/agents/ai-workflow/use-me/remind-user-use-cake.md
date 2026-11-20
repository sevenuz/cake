---
id: remind-user-use-cake
status: done
parent: "[[use-me]]"
created: 2026-11-13T18:39:06+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-13T20:13:21+01:00
time:
  - 2026-11-13T18:49:33+01:00/2026-11-13T18:49:33+01:00
---
# Remind the user to use cake while working with AI

- in a repo with a store, the agent suggests capturing todos, ideas and
  decisions from the chat into cake (one line, not every turn)
- when the user mentions "later", "todo", "idea": offer `cake add`
- skill rule first; a session start brief is [[session-brief]]
- done when: skill rule ✓ (13a156b) + one session where the reminder fired
- try: in a Claude session mention "we should ... later": Claude offers `cake add`
