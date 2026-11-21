---
id: sync-errors
status: done
tags: [git]
parent: "[[sync]]"
created: 2026-11-13T18:39:06+01:00
by: claude-code
commits: [0ea9fb0]
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
closed: 2026-11-21T01:13:13+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-20T23:12:46+01:00/2026-11-20T23:14:34+01:00
tokens_in: 14
tokens_out: 4787
tokens_cache_read: 1564090
tokens_cache_write: 8045
---
# Sync errors that say what failed and what to do

- user: sync stays simple, but errors must be expressive
- name the failed step (fetch, merge, push) and the git message
- hint per case: auth/SSH (ssh-agent), no network, rejected push,
  conflict (`cake sync --continue`/`--abort`)
- done when: tests for fetch failure, rejected push, conflict

## Log
- 2026-11-20 changed: a failed cake sync now says 'sync: fetch|merge|push failed: <git message>' plus a hint: SSH key or credentials (ssh-add, ssh -T), no connection (changes stay committed locally), remote URL not reachable (git -C .cake remote -v), push rejected because the remote moved (run cake sync again) or refused by a hook or protected branch; a conflict keeps its --continue/--abort hint. Tests: missing remote, a remote hook refusing the push, conflict, hint texts. try: cake sync with the network off
