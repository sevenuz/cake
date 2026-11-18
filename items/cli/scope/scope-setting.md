---
id: scope-setting
parent: "[[scope]]"
created: 2026-11-18T22:30:22+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Scope: optional per-link setting

Context: a dir linked to a goal shows only that goal's subtree in ls, ready and the form; --wide shows the store.
Why: someone might want captures under the goal but views over the whole store.
Change (idea, user 2026-11-18: note only, default stays scoped): `init --parent g --no-scope`, stored as "scope": false in the link entry.
Not: a global config switch (one dir may want scoping, another not).
