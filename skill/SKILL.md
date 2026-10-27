---
name: cake
description: Track tasks, plans, docs and ideas with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, ideas, time tracking or "remember this" (project or personal).
---

# cake

One markdown file per item in `.cake/items/` (project store, own git branch)
or `~/.local/share/cake` (personal store, `-g`). Every change through `cake`
is committed automatically. Your items get `by: <agent>`, changed items record
your session in `sessions`, your commits are tagged `[<agent>]`.

## Rules

- **Always `--json`.** stdout is one JSON document, errors included.
- **Never interactive:** content via `-m TEXT` or `-m -` with a heredoc. No
  `cake edit` without `-m`, no `cake config`.
- **Only through `cake`:** never edit `.cake/` files or run git in it.
- **Plans and docs go into the store**, never as files in the repo.
- **Edit only your own items** (`by` = you). To add to the user's items,
  create a child. Status and tags of the user's items only when asked.
- **Implement only approved items** (status `todo`/`doing`) the user asked
  you to work on.
- **Short:** titles ≤ 60 chars, bodies ≤ 5 bullets, no prose.
- **No `cake sync`** unless asked. Cite items by id.

## Workflow

```
 user: cake add            idea · (user's note, never edited by you)
                             │ discuss in chat
 you: cake add -p <idea>   planned · (your note under the idea)
                             │ user: cake todo <id>          (approve)
                           todo ○  (ready if no open blockers)
                             │ cake start <id>               (timer runs)
                           doing ◐  ↺ cake stop to pause
                             │ you: cake stop, cake tag <id> review,
                             │      list the commits in chat
                           review
                             ├ user "ok"       → you: cake done → ✓
                             ├ user "change X" → back to doing
                             └ user "drop"     → you: cake drop → ✗
```

- **Ideas:** `cake ls --mine --json` shows the user's ideas and what grew under
  them. Planned (no status) children are candidates for the user to approve.
- **Plans:** one note per idea or deliverable under the idea, ≤ 5 bullets,
  `done when` included. Ordering with `--blocked-by`. Link with `[[id]]`.
- **Docs:** your reference notes are top-level and tagged `doc`.
- **Time:** start your timer on approved items you work on, stop it when you
  pause or request review. The owner of an item tracks its time.
- **Coming back:** when asked what to do next, use `cake ls --mine --json` and
  `cake ready --json`, propose what to discuss and what has priority.

## Commands

```sh
cake ls --json [selectors]              # tree: matches with children; -l adds bodies
cake ready --json                       # approved, unblocked, by priority/due
cake show <id> --json                   # one item with body
cake add --json [--todo] [-p ID] [-t TAGS] [--blocked-by IDS] [--prio N] [--due D] -m TEXT
cake edit <id> --json -m -              # replace the body (your items only)
cake edit <id> --json --prio 1 --due 2026-10-30   # 0 / - clear
cake start|stop <id> --json             # timer; start sets doing
cake todo|done|drop <ids> --json        # status
cake tag <ids> a,~b --json              # add a, remove b
cake mv <id> <parent|-> --json
cake block <id> --by <ids> --json       # unblock removes
cake rm <ids> --json
```

Ids: any unique prefix (≥ 2 chars) works. Selectors (and; `--or`): `-s TEXT`,
`--status todo,doing|done|dropped|none` (none = notes), `-t tag,~not`,
`-p PARENT`, `--mine`, `--by NAME`, `--ready`, `--blocked`, `--childless`,
`-A` (with done/dropped).

## Output

- `{"v": 1, "items": [...]}` (after the change), `rm` → `{"v": 1, "removed": [ids]}`.
  `ls`/`ready` without `body` unless `-l`.
- Item: `id`, `title`, `body`, `status` (null = note), `tags`, `parent`,
  `children`, `blocked_by`, `blocks`, `ready`, `due`, `priority`, `created`,
  `closed`, `by`, `sessions`, `running`, `tracked_seconds`, `time`, `path`.
- Error: `{"v": 1, "error": {"code", "message"}}`, exit 1. Codes:
  `not_found`, `ambiguous_id` (use a longer prefix), `invalid_input`,
  `conflict` (unresolved sync merge: tell the user, stop writing),
  `not_published`, `git`, `other`.

## Personal store

Things not about this project: `cake -g add --json -m "…"`, find with
`cake -g ls --json -s …`.
