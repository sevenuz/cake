---
id: scope
status: task
parent: "[[cli]]"
created: 2026-11-18T16:15:24+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
---
# Scope: linked dirs with a default goal

Context: a store can be used from dirs without a .cake (stores.json `dirs`), e.g. one PhD store for gekkofs, papers and io500. Everything captured there lands at the top level, and every view shows the whole store.
Why: in ~/git/gekkofs captures should land under the `gekkofs` goal, and a session there should see and work only on that goal.
Change: as designed in [[scope-doc]].
Done when: the children are done and the PhD store runs with gekkofs linked to its goal.
