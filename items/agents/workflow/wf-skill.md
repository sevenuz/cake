---
id: wf-skill
status: review
parent: "[[workflow]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
commits: [5329b96]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h30m
time:
  - 2026-11-18T16:52:48+01:00/2026-11-18T16:53:31+01:00
---
# Skill text for layout, depth, handoff, gardening

Context: the cake skill still says bullets only, 3 levels, and files only top-level items.
Why: agents follow the skill, not the docs.
Change: rewrite the skill sections per [[wf-layout]], [[wf-depth]], [[wf-handoff]], [[wf-garden]]; session start/end steps in Chat mode read and update the goal's handoff.
Done when: the skill says all four, and one chat session used it.

comment (claude-code, 2026-11-18): also: triage renames random or sentence ids to topic ids, listed in the summary ([[ids]])

changed: skill sections Item layout (Context/Why/Change/Done when/Not, ## Log, rewrite own state), Names (random ids, triage renames only those), Tree (no depth limit, nested goals, docs split, living handoff), agent run (log lines, gardening per goal, handoff update), chat mode (session start/end with the handoff); try: read skill/SKILL.md on branch workflow
