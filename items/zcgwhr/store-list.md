---
id: store-list
status: done
tags: [cli]
parent: "[[zcgwhr]]"
created: 2026-11-13T18:28:48+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
closed: 2026-11-13T20:13:09+01:00
time:
  - 2026-11-13T18:28:48+01:00/2026-11-13T18:31:43+01:00
---
# Store list in the global config

- config `stores: {name: path | {store, dirs}}`, registered on init
  and when a project store is used first
- lookup: nearest .cake, then a store whose `dirs` hold the cwd, then -g
- `cake --store DIR init` outside a repo: plain store, registered with
  the cwd as dir (paper case)
- `cake` (morning): paper todos to the current store, questions and
  planned items from all stores, one section each
- done when: tests for lookup, registration, morning over 2 stores
- stores.json separate from config.hjson (cake writes it, keeps your hjson comments)
- commit: 45cf693
- try: cat ~/.config/cake/stores.json; for the paper: `cake --store ~/stores/paper init` inside the paper dir
