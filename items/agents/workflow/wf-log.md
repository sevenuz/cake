---
id: wf-log
status: done
parent: "[[workflow]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
commits: [8b663d7]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T17:08:38+01:00
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T16:53:31+01:00/2026-11-18T16:55:20+01:00
tokens_in: 16
tokens_out: 4250
tokens_cache_read: 1090405
tokens_cache_write: 7308
---
# Log section: cake appends under ## Log

Context: `--comment`, `set -m` (answers, `changed:`) and the review `discuss` line append to the end of the body.
Why: with [[wf-layout]] the body is the current state; events belong in `## Log`.
Change: cake appends these as dated lines under `## Log`, creating the heading when missing; old bodies stay as they are.
Done when: comment, answer, review and discuss land under `## Log`, tested, and the form still shows them.

changed: set -m (answers, review notes) and --comment append '- <date> <text>' under ## Log, the heading is added at the end when missing, later lines indented; discuss line without its own date; old bodies untouched; try: cake review <id> -m 'changed: a' on branch workflow, then cake show <id>
