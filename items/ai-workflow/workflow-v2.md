---
id: workflow-v2
status: task
tags: [plan]
parent: "[[ai-workflow]]"
created: 2026-11-13T23:04:17+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
assignee: claude-code
owner: sevenuz
---
# Workflow v2: inbox, task, review with assignees

- why: today the state is spread over status plus tags (ask, reply,
  review, triaged, ai); the form shows agent things before your own and
  approving plans is a separate, confusing step
- you only capture with `cake add`; everything else happens in `cake`
- status: inbox → note | task → doing → review → done | dropped
- fields: assignee (who does it), owner (who reviews: whoever made it a
  task; for subtasks the parent's owner), due, deadline, prio
- questions are tasks assigned to you that block someone's work, with
  options a/b/c in the text; comments and change requests are appended
  to the item's text, not new items
- AI subtasks of an AI task become tasks directly (no waiting day); you
  see them in `news` and review
- a parent with open children waits and goes to review by itself when
  all children are done
- done when: the steps below are done and one week runs on it
