---
id: wf-log
status: task
parent: "[[workflow]]"
created: 2026-11-18T16:15:08+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 2h
---
# Log section: cake appends under ## Log

Context: `--comment`, `set -m` (answers, `changed:`) and the review `discuss` line append to the end of the body.
Why: with [[wf-layout]] the body is the current state; events belong in `## Log`.
Change: cake appends these as dated lines under `## Log`, creating the heading when missing; old bodies stay as they are.
Done when: comment, answer, review and discuss land under `## Log`, tested, and the form still shows them.
