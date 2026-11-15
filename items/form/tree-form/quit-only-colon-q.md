---
id: quit-only-colon-q
status: done
parent: "[[tree-form]]"
created: 2026-11-14T20:51:28+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 8, out: 1280, cache_read: 452213, cache_write: 3797}}
closed: 2026-11-14T21:29:13+01:00
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T20:52:41+01:00/2026-11-14T20:54:42+01:00
---
# Only `:q` quits the form, `q` gives a hint

- why: pressing q by accident closes the form
- plain q no longer quits; it shows "type :q to quit" in the note line
- :q, :q!, :wq, :x keep working
- done when: q leaves the form open with the hint; test

changed: plain q shows 'q does not quit: type :q to close the form'; :q/:wq/:x quit, also on the end and empty screens; ctrl-c still quits; config keys.quit can bind a key again; scenario test for a stray q; commits: 6bebaf5; try: cake form, press q, then :q
