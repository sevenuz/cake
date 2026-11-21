---
id: config-ai-defaults
status: review
parent: "[[agents]]"
created: 2026-11-19T10:24:12+01:00
by: sevenuz
commits: [98e5da6, 0bbc375]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
time:
  - 2026-11-21T01:57:09+01:00/2026-11-21T01:57:09+01:00
---
there is still quite a bit of AI stuff in the default config at least. we might also move it into the skill part somehow? there is also no alias example. you can also not define/disable that review max of 8. btw for this max, i am not sure if i like it, it has to be at least configurable. a time threshold would be better maybe because then you can tell the agent, okey work as long as ready items exist or 30m or 1h or so, so that you can better work on other things in the meantime...

## Log
- 2026-11-20 comment (claude-code): plan (triage): split into 1) AI parts of the default config (sort_who ai, the claude triage alias) move into the skill's hooks examples, 2) an alias example in the default config, 3) the review brake: max_review is already set per store (-1 off); add a time budget for agent runs (work until ready is empty or N minutes). Done when: default config has no agent-specific commands, has an alias example, and the budget is documented in the skill.
- 2026-11-20 comment (claude-code): decided (user, 2026-11-20): the store config shows max_review with a comment (default 8, -1 off) as part of this task.
- 2026-11-21 changed: the default config names no harness: session_env only CAKE_SESSION, sort_who me, an alias example (mine, week, a ! shell alias); the Claude Code aliases, session env and hook setup moved to skill/hooks/README.md (your config keeps sort_who ai, added by me, and its own session_env); new stores get a commented .cake/config.hjson (max_review 8, -1 off, people, agents, fields), older ones via 'cake config store'; time budget for agent runs in the skill ('work 1h': stop before the next item); log add lost --agent (never read), --session has help. try: cake config store; cat ~/.config/cake/config.default.hjson
