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

Context: agreed in chat 2026-11-18. The item header gets a generic map for keys cake does not know; agent-specific code and the Claude Code hook leave the core; stats becomes declarative.
Why: cake should stay a plain tool; agents, harnesses and people add the keys they need without code changes.

Decided:
- Core keys stay: status, parent, tags, dates, prio, est, assignee, owner, blocked_by, by, commits (people use them too).
- Every other key is a free field: kept, written back in order, shown in `show` and `--json`. `sessions`, `usage`, `footprint` become free fields; existing items keep their keys as they are.
- CLI: `edit --set key=value`, `--add key=value` (lists append; numbers are summed), `--unset key`; selectors `--where key=value`, `--has key`.
- Store config `fields`: types for keys that need them (list, number, duration, date) and derived fields with a formula over number fields, computed at read time, so changed rates apply to old items too.
  ```hjson
  fields: {
    tokens_in:  { type: "number" }
    tokens_out: { type: "number" }
    wh:    { derive: "tokens_in * 0.00025 + tokens_out * 0.002" }
    co2_g: { derive: "wh * 0.4" }
  }
  ```
- Formulas via the `evalexpr` crate (measured +145 KB release, about 6% of 2.5 MB; user leaned crate). Only number results are used; a formula error shows as `?` in stats and once as a warning.
- Time merging: `stats.merge_by: ["session", "by"]`; overlapping timers of items with the same value of the first present key count once. Replaces the hardcoded session merge.
- Stats declarative only: config `stats.sections`, each row a label plus an aggregate over a field or derived field: `sum`, `avg`, `per_done`, `top N`, `heat`. Built-ins stay (your time, done, rhythm, flow); the agents and footprint sections become config, shipped as an example in the skill.
  ```hjson
  stats: { sections: [ { title: "agents", rows: [
    { label: "tokens", sum: "tokens_in + tokens_out" }
    { label: "tokens per done item", per_done: "tokens_in + tokens_out" }
    { label: "top items", top: 5, by: "tokens_in + tokens_out" }
  ]} ] }
  ```
- Harness adapters are scripts in the skill dir (Python), one per harness (`hooks/claude-code.py` now, local models later). They read the harness's own log and only call cake (`edit --add tokens_in=…`, `--set session=…`); pending state for running timers in their own state file. The keys they write are a contract written in the skill.
- Period share: a summed field counts by the share of the item's time in the period, as tokens do now.

Not:
- by/commits out of the core (authorship and commit links are used by people).
- a built-in optional agents section (still AI code in the core).
- script sections in stats (slow, no charts; declarative only, user 2026-11-18).
- the hook writes finished footprint numbers (rates could not change afterwards).
- shell + jq adapter (Python reads the JSON transcripts more easily).

Edge cases: a free key that later becomes a core key → migrate.rs; `--add` on a non-number, non-list key → error; a derived field of a missing key → 0; merge_by key absent on all items → per item.
