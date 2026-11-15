---
id: agent-metadata-field
status: task
parent: "[[we-want-make-cake]]"
created: 2026-11-14T22:57:07+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac]
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
