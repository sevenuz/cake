---
id: fields-config
status: done
parent: "[[fields]]"
blocked_by: ["[[fields-map]]"]
created: 2026-11-18T16:46:24+01:00
by: claude-code
commits: [695dd95]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T21:55:13+01:00
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T17:22:38+01:00/2026-11-18T17:26:04+01:00
tokens_in: 56
tokens_out: 22970
tokens_cache_read: 6533468
tokens_cache_write: 32742
---
# Fields: types and derived formulas in config

Why: stats needs numbers and formulas without knowing what they mean.
Change: config `fields` with types and `derive` formulas (evalexpr); `--add` sums numbers; derived values computed at read time, errors as `?` plus one warning.
Done when: the hjson example of [[fields-doc]] loads and computes, tested.

## Log
- 2026-11-18 changed: store config fields: {key: {type: number|list|date|duration|text}} checks --set/--add, list types append from the first value; {derive: formula} computes at read time into json fields, never stored, derived fields may use each other in any order (evaluated until stable), missing keys are 0, strings and assignments work; a broken formula gives ? and one stderr line; release binary 2580712 -> 2717944 bytes; try: put the example of fields-doc into .cake/config.hjson, cake show <id> --json
