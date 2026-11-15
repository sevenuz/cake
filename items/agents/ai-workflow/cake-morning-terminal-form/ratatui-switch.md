---
id: ratatui-switch
status: done
tags: [cli]
parent: "[[cake-morning-terminal-form]]"
created: 2026-11-13T19:51:15+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
usage: {claude-opus-5-5: {in: 52, out: 31887, cache_read: 7831727, cache_write: 42388}}
closed: 2026-11-13T20:13:32+01:00
time:
  - 2026-11-13T19:51:15+01:00/2026-11-13T19:58:30+01:00
---
# Terminal UI on ratatui, termimad removed

- why: the form needs redraw in place, f hints and panels; the dashboard
  needs sections and a pie chart; termimad brings a 3 year old crossterm
- user: switch in one go, several commits; colors stay in a theme file
- steps: theme file + ratatui, show/scroll without termimad, form TUI,
  termimad removed
- done when: cake builds without termimad, form runs as TUI, show and
  ls keep their look (theme colors)
- commits: eef986f theme, a038543 markdown+pager (termimad gone), 3cc0626 TUI form
- release binary 2.84 MB -> 1.77 MB
- try: `cake` (form), `cake show ai-workflow` (markdown), colors in ~/.config/cake/theme.hjson
