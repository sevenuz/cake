---
id: comment-author-agent
tags: [doc, bug]
parent: "[[cli]]"
created: 2026-11-14T00:48:32+01:00
by: claude-code
sessions: [09ab2c5e-817c-4232-99aa-cd89bfb2b546, eb1412a2-56ac-4478-9717-6ebac99000ac]
---
# Comments by agents are signed with the user's name

- why: `cake comment` run by an agent writes "comment (sevenuz, …)"; the user can't tell agent remarks from their own
- cause: comment uses the human user (CAKE_AUTHOR, git user.name, $USER), not the author that new items use (agent first)
- seen: comments on what-does-do-assign, how-do-i-set, i-lost-my-input (2026-11-14); the later two say "(from claude-code)" by hand
- fix idea: sign comments with the author (agent, else user), like `by:` on new items
