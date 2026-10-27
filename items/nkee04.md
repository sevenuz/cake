---
id: nkee04
tags: [design]
created: 2026-10-27T12:28:19+01:00
by: claude-code
---
# Design: CLI and --json

clap 3 → 4. Selector flags currently repeated in 6 subcommands → one
`#[command(flatten)] SelectorArgs`.

```
cake                         TUI dashboard (static summary if stdout not a tty)
cake add [-m TEXT|-] [-t TAGS] [-p PARENT] [--todo] [--due D] [--prio N]
         [--blocked-by IDS]
cake edit <id>               $EDITOR on the item file
cake start <id> | stop [id] | done <id> | drop <id> | todo <id>
cake tag <sel> +a -b
cake mv <id> <parent|->      re-parent / detach
cake block <id> --by <ids>   add blockers (cycle check like re-parent)
cake unblock <id> --by <ids>
cake ready                   open + unblocked, sorted by priority, due
cake rm <sel>
cake ls [selectors] [-l] [-r]
cake show <id>               rendered markdown
cake log [--since 7d]        time entries
cake stats [--since …]       numbers behind the dashboard
cake sync [--publish [REMOTE]|--continue|--abort]
cake init [--global] [--remote]
cake config
```

Selectors (existing + new): ids, `--tag`/`~tag`, `--parent`, `--status`,
`--status none` (notes), `--by NAME`, `--blocked`, `--ready`, `--before/--after`, `--started/--stopped`, `--due-before`,
`-s TEXT` (substring in title/body), `-r` recursion, `--or`.

Non-interactive guarantees (needed for agents): no `$EDITOR` when
`-m` (`-m -` = stdin) is given, when `--json` is set, or when
stdin is not a tty.

### `--json`

Global flag. Rules:

- stdout = exactly one JSON document; nothing else (no color, no hints,
  debug output on stderr). Top-level `"v": 1` for schema versioning.
- `ls` / `ready` / `show <id>` / mutating commands → `{"items": [...]}`
  (the items after the change, `ls` in list order); `rm` → `{"removed":
  [ids]}`; `init` → `{"store"}`; `sync` → `{"synced": true}` /
  `{"published": remote}` / `{"aborted": true}`; `show <file>` → `{"path",
  "content"}`. Later: `stats` / bare `cake --json` → stats object.
- Errors → `{"error": {"code": "not_found", "message": "…"}}` on stdout,
  exit code 1. Stable codes: `not_found`, `invalid_input`, `conflict`
  (unresolved merge), `not_published` (sync without upstream), `git`,
  `other`. Later with id prefixes: `ambiguous_id`.
- `--json` never opens an editor: `add` without `-m`, `edit`, `config` →
  `invalid_input`.
- Item JSON = frontmatter fields + `title`, `body`, `children` (derived),
  `blocks` (derived), `ready` (bool), `tracked_seconds`, `running` (bool),
  `path`, `extra` (unknown keys).
