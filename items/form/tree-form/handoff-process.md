---
id: handoff-process
status: done
parent: "[[tree-form]]"
created: 2026-11-14T13:17:02+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-14T20:20:54+01:00
assignee: sevenuz
owner: sevenuz
---
# How do we hand off between sessions?

- why: the handoff note (store item handoff) went stale after one session; a new session read it first and would have started from a wrong picture; the memory pointer to it is removed (2026-11-14)
- to discuss: what a handoff holds (only what the store does not: how the user works, build, open threads) vs. what the items already say; who writes it and when (end of session, after big changes); one note or a tag (`handoff` on the plans in progress, like tree-form now); how a session finds it (skill, memory, session start hook qzf5dz)
- related: also-handoff-needs-be (user: handoff covers only what the store does not, links the session's items; can sessions be queried if agent fields move into metadata, see we-want-make-cake)
- options to start from:
  - a) no handoff note; plans in progress carry the tag handoff, the session start hook lists them with their open children
  - b) one handoff note, rewritten at the end of each session, short, linking items
  - c) both: tag for open work, a short note for how the user works and how to build
- done when: we agreed a way, it is written as a rule note, and the old handoff item is rewritten or removed

answer: a) no handoff note; plans in progress carry the tag handoff, the session start hook lists them with their open children
