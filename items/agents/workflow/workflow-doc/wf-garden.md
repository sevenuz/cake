---
id: wf-garden
parent: "[[workflow-doc]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Gardening per goal

Context: gardening only moved top-level items under a goal, so a dump on top of `gekkofs` stayed there.
Decided (user, 2026-11-18, option 4a):
- Walk every goal, deepest first.
- Per goal: direct children that are not goals, plans, docs or handoffs move into the best fitting subgoal or plan; nothing fits → stays, named in the summary.
- The agent moves itself, only the parent changes, every move listed in the summary so the user can undo it.
- Scoped session (linked dir): only that subtree; store-wide triage: all goals.
- Overlaps still only asked ("Merge a into b?"), never merged.
