---
id: add-output-parent
status: task
parent: "[[cli]]"
created: 2026-11-19T14:01:38+01:00
by: sevenuz
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
---
i added a new item:

jul@jul-ux303ub:~/git/gekkofs(jathenst/pointertrick)$ cake add -t goal -m"# Design doc of current GekkoFS" --due 11.12
xurkat · inbox · #goal · by sevenuz · created 2026-11-19 13:59 · due 2026-12-11 · parent gekkofs

Design doc of current GekkoFS
xurkat → gekkofs





what i dont like here is the last line, the relation. its already in the header and it looks like it would be content of the note

## Log
- 2026-11-20 comment (claude-code): plan (triage): plain cake add output ends after the header and body; the 'id → parent' line goes, the parent stays in the header; done when: the CLI test for add shows no relation line.
