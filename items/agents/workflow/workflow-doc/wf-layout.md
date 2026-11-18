---
id: wf-layout
parent: "[[workflow-doc]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Item layout: current state on top, log below

Context: bodies grew by appending (plan, then `changed:`, then `discuss`), so the current state was hard to find; bullet-only bodies lost the context.
Decided (user, 2026-11-18, option 1a):
- Fields in this order, short full sentences, bullets only for real lists:
  - `Context:` 1–3 sentences, readable cold days later
  - `Why:` the problem in plain words
  - `Change:` what gets done
  - `Done when:` a checkable state
  - `Not:` rejected options or out of scope (optional)
- Notes (docs, rules, goals) use Context/Why/Change where they fit; docs may add `Decided:`, formats, edge cases, examples.
- `## Log` at the end: one dated line per event (`- 2026-11-18 changed: …; try: …`, `- 2026-11-18 comment (sevenuz): …`, `- … discuss (…): …`, `- … answer: …`).
- Everything above `## Log` is the current state: the agent rewrites it on its own items instead of appending.
- The user's items: body never touched; agents only add log lines and children.
Rejected: bullets only (lost context); appending history to the body (current state unreadable).
Example:
```
# Merge driver for sync conflicts

Context: two clones editing one item end in a git conflict that cake cannot read.
Why: sync stops and the user has to fix markdown by hand.
Change: a git merge driver that merges front matter per key and bodies per line.
Done when: the two-clone test syncs without a conflict marker.

## Log
- 2026-11-18 changed: driver in sync/merge; try: cake sync in both clones
```
