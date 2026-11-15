---
id: key-line-narrow
status: review
parent: "[[tree-form]]"
created: 2026-11-14T13:14:14+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
time:
  - 2026-11-15T17:36:37+01:00/2026-11-15T17:39:58+01:00
---
# Key line cut in narrow panes

- why: the key line under the box is cut at the right below ~100 columns (e.g. "? keys" lost), so the help key is not visible
- what: shorter labels or two lines when narrow; ? keys always visible at the end
- done when: at 60 and 80 columns every module shows its key line complete or ending with ? keys

changed: below 100 columns the key line takes two rows (always, so the box does not jump), whole groups per row; still too long: groups cut before the end with …, ? keys always last; test at 60/80 columns; commits: 1fb0799; try: cake in a 70-column pane
