---
id: debug-git-cmds
status: task
parent: "[[cli]]"
created: 2026-11-19T11:45:33+01:00
by: sevenuz
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
---
-d debug output should show git commands before executing them

## Log
- 2026-11-20 comment (claude-code): plan (triage): with -d every git command cake runs is printed to stderr before it runs (program, args, working dir); done when: a test sees the command line on stderr with -d and nothing without.
