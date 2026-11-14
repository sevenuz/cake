---
id: form-test-env-leak
status: task
parent: "[[tree-form]]"
created: 2026-11-14T22:49:48+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Form tests pick up the developer's git name and session

- why: the scenario test (tests/form_scenarios.py) runs cake with the developer's environment; owner comes from the git user name and the agent session id from CLAUDE_CODE_SESSION_ID, so results depend on who runs it
- seen: a review item got owner sevenuz instead of jul and failed to show in the review module; worked around by setting the owner by hand
- fix: the test env sets the user explicitly and removes session and agent variables (GIT_*, CLAUDE_CODE_SESSION_ID, CAKE_SESSION)
- done when: the test passes the same with another git user and inside a Claude session; the manual owner workaround is gone
