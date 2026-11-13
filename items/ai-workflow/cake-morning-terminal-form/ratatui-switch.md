---
id: ratatui-switch
status: doing
tags: [cli]
parent: "[[cake-morning-terminal-form]]"
created: 2026-11-13T19:51:15+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
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
