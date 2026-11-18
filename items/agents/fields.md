---
id: fields
status: task
parent: "[[agents]]"
created: 2026-11-18T16:29:39+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
assignee: claude-code
owner: sevenuz
---
# Fields: free header keys instead of agent fields

Context: cake has hardcoded agent fields (sessions, usage, footprint) and a Claude Code hook in its core, though it should not depend on AI.
Why: cake should stay a plain tool; agents and people add the keys they need without code changes.
Change: as designed in [[fields-doc]]; replaces [[agent-metadata-field]].
Done when: the children are done and no agent-specific key or hook is left in the core.
