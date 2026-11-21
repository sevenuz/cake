---
id: tui-filter-design
tags: [doc]
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
owner: sevenuz
---
# Filter and search: one query for dashboard and form

Context: proposal from the TUI consistency audit (2026-11-21), not decided yet. Today the form has `:filter <tag>` (one tag, dropped silently in a module without matches, cleared only by `:filter` alone), ALL `f` cycles open → +notes → +closed, `#` toggles tags; the dashboard has nothing; there is no text search.
Why: filtering differs per screen, gets lost between modules and cannot search text.

Proposed:
- One query, one prompt, everywhere; kept for the whole session. Dashboard → form passes `--filter "<q>"` (with `--at <id>`); back on the dashboard it stays.
- A search is a filter (narrows), not a jump: n/p keep "next/previous open item" and walk the matches. No vim n/N (N is note, p previous).
- Syntax, AND between tokens: plain words (title, id, body); `#tag`, `~#tag`; `@me`, `@ai`, `@name`, `@-` (nobody); `is:inbox|task|doing|review|note|done|dropped|open|closed`; `p1`..`p3`; `due:7d`; `late`.
- Keys: `/` opens the prompt on the last row like `:`, prefilled, narrowing while typing; `<cr>` keeps, `<esc>` in the prompt restores the previous query. `f` = the scope cycle as an `is:` token (visible, clearable). `<esc>` in normal mode with a query clears it; a second `<esc>` does what it does today; the dashboard never quits on `<esc>`. `:filter` stays as alias setting the query.
- Header: query in the tag color plus counts, e.g. `inbox · repo · /venue #doc @me  1 of 4`; tabs show `INBOX 1/2`; on the dashboard the same chip in the tab line.
- Matches drawn normally, parents as dim context (as the tree does now), matched words underlined.

Edge cases:
- 0 matches: the module stays reachable, shows "no match for /x · <esc> clears", the query stays; the flow (not h/l) passes it like an empty module.
- An item that stops matching after a change stays with ✓ until quit (as `changed` rows), never vanishes under the cursor.
- Pie and stats are aggregates: only `#tag` and `@who` apply (scoping the numbers), other tokens shown struck through "(not applied here)"; gantt views 2 and 3 filter rows.
- Answer and review cards: the query picks which questions show, never hides options.
- `#` tag display toggle stays independent.
- Invalid tokens (`p7`, `is:foo`): red note, the previous query stays.
- All tab / several stores: per store, counts summed.

Example: dashboard `/` `#paper @me late` `<cr>`: focus narrows to your late #paper items, tab line `/#paper @me late 2`; `<cr>` on a row opens `form mine --at draft --filter "#paper @me late"`, header `your tasks · repo · #paper @me late  2 of 5`; n/p walk the two; `<esc>` clears; h/l to answer shows "no match" instead of dropping the query.

Open questions: the syntax above, or plain words + `#tag` only first? Does `/` narrow live (cost in big stores like phd)?

## Log
- 2026-11-21 comment (claude-code): decided (user, 2026-11-21): the full syntax as proposed (plain words, #tag, ~#tag, @who, is:status, p1-p3, due:7d, late); period keys are < and > (see tui-view-keys)
