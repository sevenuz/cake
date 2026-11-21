---
id: review-round-summary
tags: [doc]
parent: "[[design]]"
created: 2026-11-21T03:21:33+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
---
# Code review round 2026-11-21: what changed

Context: subagents reviewed core, CLI and TUI code for structure (thermo-nuclear review), comments and TUI consistency; the code findings were implemented directly, the TUI consistency findings became items under [[tui-consistency]].

On main (tests pass, release binary built from 3d5fab8):
- Clippy: the char-index error in the duration parser and ~70 warnings fixed; one left in dash.rs (the TUI refactor removes it). Item [[clippy-clean]] in review.
- Core refactor: store.rs split into store/ (graph, score, archive, tests); one parent per item (parent()) and one cycle-safe ancestors walk; selector and/or matching from one criteria table; blockers and done medians cached until the next change; timer intervals typed; session log times as numbers, sorted by time; field types checked when the config loads (an unknown type in a store config now fails at once); migrate cleanup; golden-file tests for the item header.
- CLI refactor: lib.rs split into cli.rs, alias.rs, agent.rs, session.rs; run() a short pipeline; no debug closure; edit and add carry all options; commands own their bodies; agent rules in one policy; config `agent_commands` replaces the hard-coded triage/work hint (your config got `agent_commands: ["triage"]`); bug fixed: `cake --fields x <alias>` expands the alias; `--status ~done` is now an error.
- Comments: drift fixed in README, hook docstrings, test sandbox doc and the CLI files.
- Merge note: the core branch started 22 commits behind main; the rebase dropped -t a/b, --along and --new-for from the selector table, restored by hand in 4f553e9 and smoke-tested.

- TUI refactor (merged 2026-11-22, 3d5fab8): shared src/tui helpers; the dashboard survives a rebuilt binary (a failing cake call shows a note); dashboard rows cached, stores reread only on a disk change or once a minute; src/dash/ split, Overlay an enum of Pie, Stats, Gantt; theme colors parsed once; form: one Module enum, one key table for key line and handling, split into screen.rs, card.rs, change.rs, pure module navigation; form changes run in process (src/actions.rs: one store session and one commit per change instead of 2-3 cake calls); the form reads Store/Item, not cake ls JSON. Clippy: no warnings.
- Check in the terminal: running timers and clock views may lag up to a minute; leaving and reopening an overlay resets the pie measure and gantt view; answers saved with the option's real letter; :all still commits per child.
Skipped on purpose: serde item header (byte format), archive unification, score weights still global.

