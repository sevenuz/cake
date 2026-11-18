---
id: fields-config
status: task
parent: "[[fields]]"
blocked_by: ["[[fields-map]]"]
created: 2026-11-18T16:46:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 2h
---
# Fields: types and derived formulas in config

Why: stats needs numbers and formulas without knowing what they mean.
Change: config `fields` with types and `derive` formulas (evalexpr); `--add` sums numbers; derived values computed at read time, errors as `?` plus one warning.
Done when: the hjson example of [[fields-doc]] loads and computes, tested.
