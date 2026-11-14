---
id: quit-only-colon-q
status: task
parent: "[[tree-form]]"
created: 2026-11-14T20:51:28+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Only `:q` quits the form, `q` gives a hint

- why: pressing q by accident closes the form
- plain q no longer quits; it shows "type :q to quit" in the note line
- :q, :q!, :wq, :x keep working
- done when: q leaves the form open with the hint; test
