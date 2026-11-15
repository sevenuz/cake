---
id: yvz3rv
status: review
tags: [tui]
parent: "[[cake-dash-terminal-dashboard]]"
blocked_by: ["[[a3j36w]]", "[[dash-tabs]]"]
created: 2026-10-27T12:19:28+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:36:26+01:00/2026-11-15T23:38:08+01:00
---
# TUI live reload and editor suspend

Poll items/ mtimes once per tick so changes from another shell or Obsidian show up; a/e suspend the TUI and run $EDITOR.

changed: the dashboard re-reads all stores every 2 s (simple reload, no mtime poll: cheap at this size); a adds an item in the editor (the tab's store, all: the nearest), e edits the selected item; the TUI suspends and comes back; commits: dash-tabs + 95190b9; try: cake, a
