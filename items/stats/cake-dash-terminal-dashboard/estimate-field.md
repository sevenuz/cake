---
id: estimate-field
status: review
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 22, out: 8424, cache_read: 2169653, cache_write: 16831}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T22:37:30+01:00/2026-11-15T22:39:58+01:00
---
# Estimate field for task durations

- why: the critical path to a goal's deadline needs durations ([[dash-design]])
- cake add/edit --est 2h, shown in show and json; skill: agents estimate when they split
- fallback: median tracked time of done tasks of the same kind (agent or user)
- done when: field round-trips; fallback tested

changed: estimate field (markdown estimate: 2h30m, json seconds, show 'est 2h30m'); cake add/edit --est, edit --est - clears; store.expected(): own estimate else median tracked time of done tasks of the same kind (delegated vs the owner's own), in item json as inherited.estimate; skill: agents pass --est when splitting; also fixed tracked-time display bug (3700 s showed 1h60m40s); commits: 7e9eb35; try: cake add x --task --est 1h30m -m '# x'; cake show x
