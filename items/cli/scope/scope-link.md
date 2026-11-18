---
id: scope-link
status: done
parent: "[[scope]]"
created: 2026-11-18T16:15:24+01:00
by: claude-code
commits: [5a9ec83]
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T17:09:24+01:00
assignee: claude-code
owner: sevenuz
estimate: 2h
time:
  - 2026-11-18T16:55:24+01:00/2026-11-18T16:58:52+01:00
tokens_in: 22
tokens_out: 8754
tokens_cache_read: 1649115
tokens_cache_write: 16070
---
# Scope: dirs entries with a default goal

Why: the link has to carry the goal before anything can use it.
Change: stores.json `dirs` entries accept `{"path", "parent"}` next to plain strings; `init --parent <id>` writes it after checking the id; `rename` updates it.
Done when: old and new formats load, init and rename tested.

changed: stores.json dirs take {path, parent} next to plain paths; 'cake --store <store> init [--parent <goal>]' in a dir links it to an existing store (it used to refuse), the goal must exist, linking again replaces the goal, a dir linked to another store is an error; cake rename updates the links; try: in a scratch dir, cake --store ~/x/.cake init --parent <id>, then cat ~/.config/cake/stores.json
