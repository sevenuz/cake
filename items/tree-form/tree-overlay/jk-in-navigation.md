---
id: jk-in-navigation
status: done
parent: "[[tree-overlay]]"
created: 2026-11-14T11:59:55+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a]
closed: 2026-11-14T12:06:00+01:00
assignee: sevenuz
owner: sevenuz
---
# Close overlays with jk while navigating?

jk (config `keys.escape`) leaves text input today. In the overlay j/k move, so j moves at once and jk cannot close without delaying every j.

- a) jk only in text input; overlays close with <esc> or q (recommended)
- b) jk everywhere: j waits ~200 ms for a k before it moves

answer: a) jk only in text input; overlays close with <esc> or q
