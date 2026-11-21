---
id: dash-item-actions
status: task
parent: "[[tui-consistency]]"
blocked_by: ["[[tui-keymap]]", "[[tui-view-keys]]"]
created: 2026-11-21T02:24:44+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 24h
---
# Dashboard rows change items like the form

Context: the dashboard main list, gantt items view and path view show items but only offer e; a/A differ (dashboard a opens $EDITOR through cake add, no A; the form opens a one-line box).
Why: to start a timer on a running row you have to open the form.
Change: on every row with an id the form's item actions (t N dd c s w P u, a/A as the form's one-line box, A under the row), with undo, through the same change code as the form.
Done when: these keys work on dashboard item rows as in the form's ALL tree; tests drive one action per view.
