---
id: clippy-clean
status: task
parent: "[[cli]]"
created: 2026-11-21T02:18:13+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 1h
---
# Clippy warnings cleaned up

Context: `cargo clippy --all-targets` showed one error (a char position used as a byte index in util.rs) and about 75 warnings, so new warnings were hard to see.
Why: AGENTS.md asks for no new warnings before a commit; that only works from a clean state.
Change: fix the error and the warnings (clippy --fix for the mechanical ones, the rest by hand); allow a lint only with a reason.
Done when: `cargo clippy --all-targets` shows no warnings; tests pass.
