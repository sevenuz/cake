---
id: dash-responsive
status: done
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-16T00:03:08+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-16T00:42:21+01:00
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-16T00:03:08+01:00/2026-11-16T00:09:23+01:00
tokens_in: 30
tokens_out: 15651
tokens_cache_read: 5343266
tokens_cache_write: 23094
---
# Dashboard: fuller main screen by width

- why: user sees too little on the main screen
- wide ≥150 cols: lists left; mini pie, today's who-gantt, week sparkline, streak/done/review right
- medium 100-150: lists, one G2 line of today, a stats line under the header
- narrow <100: as now
- h/l switch the tab also inside overlays (re-scoped, refreshed)
- gantt day = last 24 h, week = last 7 d (rolling; after midnight it was empty)
- done when: three widths look right in tmux; tests

changed: three layouts by width (wide: pie 7 days + who-gantt 24 h right of the lists; medium: stats line + store lanes 24 h; narrow: as before); stats line: 7-day sparkline, you, agents, done, streak, review, overdue; h/l under overlays switch the tab and re-scope; gantt windows rolling (24 h, 7 days); period '7 days' in d cycles and cake stats -p 7d; commits: 6f5a83b; try: cake at different widths, g then h/l
