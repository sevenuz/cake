---
id: fields-map
status: review
parent: "[[fields]]"
created: 2026-11-18T16:46:24+01:00
by: claude-code
commits: [5fda1ab]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
usage: {claude-opus-5-5: {in: 48, out: 13203, cache_read: 4191015, cache_write: 27840}}
assignee: claude-code
owner: sevenuz
estimate: 3h
time:
  - 2026-11-18T16:58:55+01:00/2026-11-18T17:04:57+01:00
---
# Fields: keep unknown header keys

Why: everything beyond the core keys must survive read and write.
Change: the item parser keeps unknown keys in an ordered map, writes them back in place; `show`/`--json` include them; `edit --set/--add/--unset`; `ls --where/--has`.
Done when: round trip keeps order and values, the CLI flags are tested.

## Log
- 2026-11-18 changed: unknown header keys were already kept; new: edit --set key=value (numbers, true/false typed), --add (numbers summed, lists appended, new text key becomes a list), --unset; core keys refused; ls --where key=value (lists: contains), --has key; json shows them as fields (was extra); skill commands list them; try: cake edit <id> --set venue=SC26, cake ls --has venue
