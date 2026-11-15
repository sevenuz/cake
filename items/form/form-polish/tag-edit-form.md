---
id: tag-edit-form
status: doing
parent: "[[form-polish]]"
created: 2026-11-15T23:19:47+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T23:19:47+01:00/
---
# Tree: set tags in the form

- why: user asked how to set tags when :tag filters
- :tag doc,~old adds and removes (same syntax as cake edit --tag), on items of the module (context rows: no)
- the filter moved to :filter doc (:filter clears); undo restores tags
- done when: scenario test adds and removes a tag