---
id: tree-edit-line
status: task
parent: "[[tree-form]]"
blocked_by: ["[[tree-widget]]", "[[edit-line-fields]]"]
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, cce3e92b-a15e-4436-ae00-939981d6ff10]
usage: {claude-opus-5-5: {in: 1, out: 386, cache_read: 71621, cache_write: 478}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T12:16:16+01:00/2026-11-14T12:16:16+01:00
---
# Edit line under the hovered entry

- why: values are set in table columns now; the user wants to see and set them right under the item they are on
- what: under the hovered editable row one line `status · who:ai · prio – · due – · deadline –`; current values filled in
- keys: the existing value keys (t/n/dd/x, w, p, :due, :deadline) write into the line; no field cursor, the line only shows the values
- sort: who starts with config `sort_who` (default "ai") when an item becomes a task
- done when: values set from the line land in the store; sort_who in config template; test with sort_who "-"

changed: edit line under the cursor row in sort: status · who · prio · due · deadline, set values in the hint color, unset as –; t/n/dd/w/p and :due :deadline :who :prio set them; a field on an inbox item makes it a task; config sort_who (default ai, - nobody, me, a name) in the template; scenario test with sort_who -; commits: 21ed863; try: cake form sort, t on an inbox item: task ai

comment (sevenuz, 2026-11-14): that needs to be discussed

comment (claude-code, 2026-11-14): decided in chat: dropped, the details box shows the fields ([[tree-rows-design]])
