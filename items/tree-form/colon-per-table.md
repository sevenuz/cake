---
id: colon-per-table
status: review
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
usage: {claude-opus-5-5: {in: 16, out: 6050, cache_read: 1072647, cache_write: 9398}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:54:49+01:00/2026-11-14T20:57:18+01:00
  - 2026-11-14T22:44:47+01:00/2026-11-14T22:45:08+01:00
---
# `:` commands per table, also in review

- why: in review the `:` hint lists due/who/prio but they answer "unknown command"
- the hint lists only the commands of the current table
- review also accepts due, deadline, who, prio (edits on the reviewed item)
- done when: every command in a table's hint works there; test per table

changed: each module has its own : commands; review, mine, news: due, deadline, who, prio; inbox also move; answer only w and q; the dim hint after : lists exactly those; unknown commands say which ones work here; :q now also works on context rows; tests: hint per module, scenario :prio 2 in review; commits: bdbfc6b; try: cake form review, then :prio 2

comment (claude-code, 2026-11-14): the : line goes to the very bottom of the screen like vim, not into the detail box

changed (comment): the : line with its hint and the cursor is on the terminal's last row like vim, no longer in the details box; own words and change requests stay in the box; test checks the last row, checked in tmux; commits: 2e65093; try: cake form, :
