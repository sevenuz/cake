---
id: store-global-flag
status: task
parent: "[[cli]]"
created: 2026-11-19T13:42:04+01:00
by: claude-code
sessions: [59209220-8a77-4046-9145-aa74c0881358]
assignee: claude-code
owner: sevenuz
estimate: 1h
---
# --store and -g after the subcommand and in its help

Context: `--store` and `-g` are top-level options of `cake` (src/lib.rs, the Cli struct). clap shows them only in `cake --help` and accepts them only before the subcommand: `cake --store ~/git/work init` works, `cake init --store ~/git/work` fails with "unexpected argument '--store'". The help of `init` mentions `--store` twice without listing it.
Why: the option `init` needs most is invisible in its help and fails in the natural position.
Change: mark both as `global = true`, like `--json`. The early argument scan that loads the store's aliases before clap runs (guess_root) has to find them after the subcommand too, skipping the value of `-m` and stopping at `--`, so a message like `-m "--store"` is not taken for the option.
Done when: `cake init --help` lists `--store` and `-g`, `cake init --store <dir>` links, an alias from that store's config is found with `--store` after the subcommand, tests cover both.
