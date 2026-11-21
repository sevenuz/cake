---
id: tui-wording
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 3h
---
# TUI wording, dates and colors the same everywhere

Context: the module color depends on its position (review tab ok-colored, its title code-colored; colors shift when dump is inserted); "your own" in the ALL tree means written by you, elsewhere assigned to you; dates as 2026-11-23, 23.11. and 11-23, durations as 3:00 and 3h, the user as their name and as me, the deadline twice on focus rows; the path view shows ids where everything else shows titles; empty states phrased eight ways; overlay titles in different orders; key-line labels and their order differ per module (c is "ok" in review, "done" elsewhere; ":due" with an empty label); the ALL header count includes notes the default view hides.
Change: one color per module name; "mine" = assignee everywhere (or the legend says "written by you"); one date helper (dd.mm.), one duration helper (3h, 3h20), "me" for the user; titles in the path view; empty states lowercase, dim, what is empty plus the key that changes it; overlay titles "<view> · <period> · <measure> · <scope>"; key lines in a fixed order t N c dd s w P with one label per key; counts match what is shown.
Done when: a screen walk shows one style; snapshot tests of the changed strings.
