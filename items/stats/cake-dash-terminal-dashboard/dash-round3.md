---
id: dash-round3
status: done
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-16T00:42:21+01:00
by: claude-code
commits: [36fcb8a]
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 55, out: 19139, cache_read: 10895422, cache_write: 29819}}
closed: 2026-11-16T02:25:18+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T00:42:21+01:00/2026-11-16T00:51:09+01:00
---
# Dashboard: q per layer, work day, form keys, queue fields

- q closes one layer everywhere (overlay, form, dashboard); :q stays
- today = work day: from the first activity after a break of 5 h or more
- F opens news, inbox, answer, review; T the ALL tree; D dump
- queue and tree rows show who, prio, dates and the estimate
- done when: all four work; tests

changed: q closes the form (back to the dashboard), :q stays, overlays close with q; header: since <start of work day> (first activity after a 5 h pause; 'a new day' when idle 5 h); F news/inbox/answer/review, T tree (cake form all), D dump; queue/focus/running rows show the tree fields (who, prio, est, dates), tree rows show the estimate (~ = history); try: cake
