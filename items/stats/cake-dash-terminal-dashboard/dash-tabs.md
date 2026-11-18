---
id: dash-tabs
status: done
parent: "[[cake-dash-terminal-dashboard]]"
blocked_by: ["[[35xst1]]"]
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-15T23:32:45+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:20:06+01:00/2026-11-15T23:26:17+01:00
tokens_in: 22
tokens_out: 16410
tokens_cache_read: 3043414
tokens_cache_write: 21977
---
# Dashboard: store tabs, Now sections, bare cake

- why: one screen across projects: running, your focus, ai queue ([[dash-design]])
- tabs: all · stores (nearest selected); header totals today
- focus order: blocking answers → reviews → deadlines → own tasks; ai queue 3-5 per store
- <cr> opens the form at that module/store, back afterwards; bare cake opens it
- done when: real stores render; scenario test in tmux

changed: new src/dash.rs on bare cake (cake form unchanged): tabs all + one per store (nearest selected), header totals today (you, agents, tokens, CO2e); sections running (timers), your focus (answers that block agents → reviews with limit note → inbox → deadlines/due within 7 d → your top 3 ready tasks), ai queue (top 5 ready agent tasks per store by score, +N, review-limit stop line); card with meta, tags, est and body; <cr> form at that module and store and back, e edit, F whole form, q quit; re-reads every 2 s; unit tests + tmux scenario; commits: 1dd2b24; try: cake
