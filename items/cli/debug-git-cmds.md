---
id: debug-git-cmds
status: review
parent: "[[cli]]"
created: 2026-11-19T11:45:33+01:00
by: sevenuz
commits: [3975a81, 5d89cbf]
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-20T23:14:42+01:00/2026-11-20T23:16:02+01:00
  - 2026-11-21T01:23:49+01:00/2026-11-21T01:27:17+01:00
tokens_in: 8
tokens_out: 1806
tokens_cache_read: 921407
tokens_cache_write: 3626
---
-d debug output should show git commands before executing them

## Log
- 2026-11-20 comment (claude-code): plan (triage): with -d every git command cake runs is printed to stderr before it runs (program, args, working dir); done when: a test sees the command line on stderr with -d and nothing without.
- 2026-11-20 changed: with -d every git command cake runs is printed to stderr as 'git -C <dir> <args>' before it runs; without -d nothing; try: cake -d add -m test
- 2026-11-21 discuss (sevenuz): also check the code base where other useful log parameters should be shown, like which store and what an alias is doing and so on
- 2026-11-21 comment (claude-code): chat 2026-11-21: also show with -d how the store was found and the scoped goal, alias expansion (also shell aliases), config files loaded, parsed selector and id prefix resolution; no hook or agent env output (cake stays unaware of AI)
- 2026-11-21 changed: -d now also prints how the store was found (--store, CAKE_STORE, -g, the .cake here or above, linked dir, global fallback), the scope goal of a linked dir, the user and store config files, alias expansion (shell aliases with their sh command line), the selector in one line and id prefixes (id ls-id → ls-id-header); color only on a terminal; nothing about hooks or agents. try: cake -d ready; cake -d show ls-id
