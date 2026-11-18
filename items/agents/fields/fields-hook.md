---
id: fields-hook
status: task
parent: "[[fields]]"
blocked_by: ["[[fields-map]]", "[[fields-stats]]"]
created: 2026-11-18T16:46:24+01:00
by: claude-code
commits: [7ecb6f2]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T17:26:07+01:00/2026-11-18T17:29:22+01:00
tokens_in: 18
tokens_out: 10996
tokens_cache_read: 2272589
tokens_cache_write: 19261
---
# Fields: Claude Code adapter as a script

Why: the transcript hook only fits Claude Code and lives in the core (`cake usage claude-hook`).
Change: `hooks/claude-code.py` in the skill dir writes tokens_in/out and session via `cake edit`; pending state in its own file; the key contract in the skill; `cake usage` removed.
Done when: one chat session records tokens through the script, the command is gone.

## Log
- 2026-11-18 progress: cake log add and hooks/claude-code.py done (7ecb6f2), tested in a sandbox (tokens per item once, chat and agent log lines); open: switch the Stop hook in ~/.claude/settings.json to the script and remove cake usage claude-hook, only together with fields-stats (stats still reads the old usage key)
