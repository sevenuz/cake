---
id: scope-add
status: review
parent: "[[scope]]"
blocked_by: ["[[scope-link]]"]
created: 2026-11-18T16:15:24+01:00
by: claude-code
commits: [18391ab, e47863f]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 1h
time:
  - 2026-11-18T17:12:47+01:00/2026-11-18T17:13:36+01:00
---
# Scope: captures land under the default goal

Why: a dump in ~/git/gekkofs should already sit under `gekkofs`.
Change: `add` and form capture without `-p` use the default; `-p -` means top level; output shows `→ <goal>`; missing goal → top level with a warning.
Done when: tested from the dir, a subdir, with `-p -`, `-g` and a missing goal.

## Log
- 2026-11-18 changed: add (and the form's a, which calls add) without -p uses the goal of the linked dir when that dir links to the store in use; -p - is the top level; -g ignores it; a gone goal adds at the top level with a warning; stderr says '<id> → <goal>'; skill mentions it. Differs from scope-doc: an explicit --store to the same store keeps the default (the form calls cake with --store); try: cd a linked dir, cake add -m x
- 2026-11-18 comment (claude-code): note (claude-code): one run of id_prefixes_work_everywhere_and_ls_shows_short_ids failed once, 3 reruns passed; flaky, cause not found yet
