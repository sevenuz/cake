---
id: agent-metadata-field
status: dropped
parent: "[[ai-free]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657]
closed: 2026-11-18T16:29:39+01:00
assignee: ai
owner: sevenuz
---
# One agent field for sessions, usage, footprint

- why: AI-only fields sit next to the user's fields in every item; cake should stay agent-independent
- decided (user, 2026-11-14): frontmatter key `agent:` holding `sessions`, `usage`, `footprint`; `by` stays top level (people have it too)
- migration: cake moves the old flat keys into `agent:` when it reads an item (migrate.rs), writes the new form
- new selector `--session <id>`: items whose `agent.sessions` contains it
- `--json` output: same names nested under `agent`; skill and form updated
- done when: old items read fine, new ones write `agent:`, `cake ls --session X` works; tests

superseded by [[fields]]: free header keys instead of one agent field (user, 2026-11-18)
