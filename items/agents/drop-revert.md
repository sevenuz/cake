---
id: drop-revert
status: task
parent: "[[agents]]"
created: 2026-11-16T02:25:18+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Dropped items with commits get reverted

- why: drop only closes the ticket; its commits stay in main
- decided (user, 2026-11-16): an agent creates a revert task (assignee the agent, owner the user) for dropped items with commits; cake writes nothing itself
- skill rule in the agent run; tag the dropped item reverted when the revert task exists
- done when: skill rule written