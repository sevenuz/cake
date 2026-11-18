# Working on cake

Rules for agents developing this repo. How to *use* cake (items, states,
triage) is in the cake skill, `skill/SKILL.md`; keep that file free of
rules about developing cake, it is used on other machines and stores too.

## Principles

- cake never calls an AI API and must be fully useful without AI. Agents
  drive it from outside through the CLI and the skill.
- No agent- or harness-specific code in the core: agent data are free
  header fields (`edit --set/--add`), harness adapters are scripts next to
  the skill, stats for them are config.
- Plans, designs and findings go into the cake store of this repo
  (`.cake/`, branch `cake/main`), never as files in the repo.

## Git

- Commit on `main`, linear history: no feature branches, no merge commits.
  Push only when asked.
- Something landed on a side branch anyway: cherry-pick it onto `main`,
  then fix the item's refs with `cake edit <id> --commit ~old,new`.
- Commit messages: what changed, in one line; body only when the why is
  not obvious.

## Before a commit

- `cargo test` passes (the scenario test takes about a minute) and
  `cargo clippy --all-targets` shows no new warnings. Run the tests before
  `git commit`, not chained after it.
- New behavior gets a test: unit tests next to the code, CLI behavior in
  `tests/commands.rs` (an `Env` gives each test its own config, data and
  store dirs).

## The live binary

- `~/.local/bin/cake` links to `target/release/cake`: `cargo build
  --release` changes the cake the user and their agents run right away,
  including the store this repo's items live in. Build release only from a
  state that passes the tests.

## Checking the TUI

- Try form and dashboard changes in a throwaway store with its own
  `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and git identity, never in the
  user's stores.
- tmux sessions for that get unique names (`cake-test-$$-<n>`); stop if
  `tmux new-session` fails; only send keys to and kill sessions you just
  created.
- Show screens as text in the chat when needed; no rendered web pages for
  UI review, the user judges the form in the terminal.
