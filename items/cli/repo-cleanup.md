---
id: repo-cleanup
status: done
parent: "[[cli]]"
created: 2026-11-19T10:12:26+01:00
by: sevenuz
commits: [b90fd35]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-21T01:12:33+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-20T23:11:33+01:00/2026-11-20T23:12:32+01:00
---
we still have cake.md and todo.md in the repo root. remove

## Log
- 2026-11-20 comment (claude-code): plan (triage): delete cake.md and todo.md from the repo root after checking nothing links to them; done when: both are gone and tests pass.
- 2026-11-20 changed: cake.md (the old single-file store) and todo.md (old notes on the stash workflow) removed from the repo root; nothing linked to them, git history keeps them; try: ls
