---
id: review-talk
status: review
parent: "[[form-polish]]"
created: 2026-11-16T00:42:21+01:00
by: claude-code
commits: [36fcb8a, 235fe18]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Review: t sends it back to talk about

- why: besides ok, drop and change, the user wants to rediscuss a topic
- t: back to task, tag discuss, assignee the user; listed in the review card options
- dashboard focus: 'to discuss' line; skill: agents bring up #discuss items first in chat
- done when: t works in the form; dashboard shows it; skill rule

changed: review t (also option 3 in the card): back to task, assignee you, #discuss, a 'discuss (you, date)' line appended, the work so far stays above; i is labelled 'change something: the agent reworks it'; dashboard focus shows '# N to discuss'; skill: bring discuss items up first, never rework unasked; scenario test; try: cake form review, t

rework (user): t and i merged: review has ok (c), drop (d) and 'not ok yet' (i or option 3): a text box for the doubts (empty: just talk); the item goes back to the user as task #discuss with 'discuss (user, date): doubts'; the agent brings it up in chat; skill updated
