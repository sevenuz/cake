---
id: fields-map
status: task
parent: "[[fields]]"
created: 2026-11-18T16:46:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
estimate: 3h
---
# Fields: keep unknown header keys

Why: everything beyond the core keys must survive read and write.
Change: the item parser keeps unknown keys in an ordered map, writes them back in place; `show`/`--json` include them; `edit --set/--add/--unset`; `ls --where/--has`.
Done when: round trip keeps order and values, the CLI flags are tested.
