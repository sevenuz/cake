---
id: wf-handoff
parent: "[[workflow-doc]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Handoff: one living note per goal

Context: handoffs were written per session and stacked up (the dropped `handoff`); `session-note` would have repeated that.
Decided (user, 2026-11-18, option 3a):
- One note `<goal>-handoff`, tag `handoff`, child of its goal; created on the first session that leaves open threads there.
- Body: 2–3 sentences, the current state and the next step; rewritten at session end, never a second note.
- Children: the open threads, half-made decisions and context no item holds yet; one note each.
- A thread graduates when concrete: moved under its goal or plan (`cake edit <id> --parent …`) and made a task or doc; the handoff shrinks by it.
- A settled thread is closed or merged into a doc.
- Session start: read the handoff of the scoped goal (see [[scope-doc]]); session end: update that same note and its children.
- History is the git log of the item; no list of touched items (found with `--session`, [[agent-metadata-field]]).
Rejected: one handoff per store (mixes projects); a note per session (stacks, repeats the store).
