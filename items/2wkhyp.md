---
id: 2wkhyp
tags: [design, git, doc]
created: 2026-10-27T12:30:10+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2]
---
# Collaboration play-through (2026-10-27)

Two clones (jul, claude) of one remote, each scenario from a synced state, both change, both sync.

| scenario | result |
|---|---|
| different items (add/add, done x/done y) | clean |
| start (status+time) vs body edit | clean |
| body edits in different paragraphs | clean |
| tag vs done, tag vs tag, done vs start, block vs tag | **conflict** (nearby frontmatter lines) |
| both tracked time on the same item | **conflict** (both append to `time:`) |
| rm vs tag | **conflict** (delete/modify) |

Same working copy (Claude Code in your repo, you in a terminal): never a merge conflict, but 2 cake processes in parallel: 16 of 40 commands failed on git's `index.lock`, 15 files stayed uncommitted (later commits only stage their own paths, so they stay uncommitted).

Side finding: a relative remote path (`../remote.git`) is resolved from `.cake/`, not the repo root. Real remotes are URLs, so only a test quirk.
