---
id: fields-doc
tags: [doc]
parent: "[[fields]]"
created: 2026-11-18T16:29:39+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Free header fields

Context: agreed in chat 2026-11-18. The item header gets a generic map for keys cake does not know; the agent-specific code leaves the core.
Decided:
- Core keys stay: status, parent, tags, dates, prio, est, assignee, owner, blocked_by, by, commits (option c: people use them too).
- `sessions`, `usage`, `footprint` become free fields; every unknown key is kept, written back in order, shown in `show`/`--json`.
- CLI: `edit --set key=value`, `--add key=value` (lists; numbers are summed), `--unset key`; selectors `--where key=value`, `--has key`.
- Store config declares types for keys that need them (list, number, duration, date) and which numbers stats sums, with a label.
- Existing items keep their keys as they are; only the special code goes (user ok, 2026-11-18).
- Replaces [[agent-metadata-field]] (one `agent` blob field): free keys instead.
- Open: what stats keeps, see chat; recorded here once decided.
Rejected: everything incl. by/commits out of the core (a: loses authorship and commit links people use); an optional built-in agents section (b: still AI code in the core).
