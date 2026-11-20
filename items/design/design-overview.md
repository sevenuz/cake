---
id: design-overview
tags: [design, doc]
parent: "[[design]]"
created: 2026-10-27T12:28:19+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, eb1412a2-56ac-4478-9717-6ebac99000ac, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
---
# Overview: goal, stages and what was left out

Goal: one CLI that is task tracker, time tracker, todo list and a queryable
markdown knowledge store (also for AI agents), synced through git without
touching the user's working tree.

Stages:

1. **Stage 1** — new storage format, git worktree sync, `--json`, htop-like TUI
   dashboard on bare `cake`, Claude skill.
2. **Stage 2** — `cake mcp` (MCP stdio server).
3. **Stage 3** — SQLite index + FTS5 full-text search.

No migration from `cake.md` / `cake.json`. The single-file formats, `-i/-o`
and the json store are deleted.

Design notes: [[pscs0p]], [[design-sync]], [[nkee04]], [[m15dd2]]; the skill lives in `skill/SKILL.md`. Open work: `cake ready`, `cake ls --status todo,doing`.
