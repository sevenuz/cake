---
id: debug-git-cmds
status: review
parent: "[[cli]]"
created: 2026-11-19T11:45:33+01:00
by: sevenuz
commits: [3975a81]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
time:
  - 2026-11-20T23:14:42+01:00/2026-11-20T23:16:02+01:00
---
-d debug output should show git commands before executing them

## Log
- 2026-11-20 comment (claude-code): plan (triage): with -d every git command cake runs is printed to stderr before it runs (program, args, working dir); done when: a test sees the command line on stderr with -d and nothing without.
- 2026-11-20 changed: with -d every git command cake runs is printed to stderr as 'git -C <dir> <args>' before it runs; without -d nothing; try: cake -d add -m test
