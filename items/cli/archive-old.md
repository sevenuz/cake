---
id: archive-old
status: review
parent: "[[cli]]"
created: 2026-11-15T18:47:52+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T19:48:44+01:00/2026-11-15T19:53:10+01:00
---
# Archive old closed items

- why: 86 of 158 items are closed and the tree keeps growing; they are needed for the time stats
- decided (user, 2026-11-15): `cake archive [--older 3m]` moves done/dropped items closed longer than that, whole closed subtrees only, to items/archive/ (same tree layout there)
- ls, ready, form and the other views do not load the archive; stats (later the dashboard) and `cake ls --archive` do
- an archived id stays taken; links and blockers to it keep working (show finds it)
- rejected: rm after 3 months (year stats lose their time), only hiding
- done when: archive moves the right items, views skip them, show and stats find them; tests

changed: cake archive [--older 3m] moves closed subtrees (notes below go along) to items/archive/, same tree layout; ls/ready/form skip it; ls --archive and show (id or prefix) load it; ids stay taken, id 'archive' reserved; rename carries archived children; reopening takes an item and its ancestors out; skill lists the command; commits: 3892a7d; try: cake archive --older 1w, then cake ls --archive
