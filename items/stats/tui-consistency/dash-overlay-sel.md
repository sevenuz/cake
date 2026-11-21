---
id: dash-overlay-sel
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 45m
---
# Dashboard overlays: own selection per view

Context: p/s/g keep the selection across kinds (pie slice 3 becomes stats scrolled 3 lines or gantt row 3); in gantt views 0/1 j/k move a hidden selection; stats j scrolls up to 80 lines past the end; the gantt footer offers "j/k row <cr> tree e edit" in views where they do nothing.
Change: reset the selection on a kind change, ignore j/k in views 0/1 and show it in their footer, clamp the stats scroll to its height.
Done when: switching views starts at the top; footers list only working keys; test.
