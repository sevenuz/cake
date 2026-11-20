---
id: sync-errors
status: doing
tags: [git]
parent: "[[sync]]"
created: 2026-11-13T18:39:06+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-20T23:12:46+01:00/
---
# Sync errors that say what failed and what to do

- user: sync stays simple, but errors must be expressive
- name the failed step (fetch, merge, push) and the git message
- hint per case: auth/SSH (ssh-agent), no network, rejected push,
  conflict (`cake sync --continue`/`--abort`)
- done when: tests for fetch failure, rejected push, conflict
