---
name: cake
description: Track tasks, plans and notes with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, time tracking or "remember this" (project or personal).
---

# cake

cake stores tasks and notes as one markdown file per item in `.cake/items/`
(project store, its own git branch) or `~/.local/share/cake` (personal global
store, `-g`). Every change through `cake` is committed to the store
automatically. Items you create get `by: claude-code` and your commits are
tagged `[claude-code]` without you doing anything.

## Rules

- **Always pass `--json`.** stdout is one JSON document, errors included.
- **Never interactive:** always give content with `-m TEXT` or `-m -` plus a
  heredoc. Never call `cake edit` without `-m`, or `cake config`.
- **Never touch `.cake/` directly:** no editing `.cake/items/*.md`, no git
  inside `.cake/`. Everything through `cake`, so commits stay consistent.
- **No loose plan files:** when a store exists, don't create `PLAN.md`,
  `TODO.md` or similar in the repo. Put them into the store.
- **Edit only your own items** (`by` is `claude-code`, or another agent name
  you run as). Never change the body, priority or due of items others created.
  To add to someone else's item, create a child: `cake add -p <id> …`.
  Status (`todo`/`done`/`drop`) and tags on others' items only when the user
  asks or agreed to the change.
- **Timers belong to the user:** don't `start`/`stop` unless asked.
- **No `cake sync`** unless the user asks (network, may conflict).
- **Cite items by id** and, after changes, list what you created or changed.

## Which store

- Project work, plans, decisions for this repo → project store (default,
  found from the current directory). It is shared with everyone who has the
  repo once published.
- Personal things ("remember for me", private todos, notes not about this
  project) → `cake -g …`. Never put personal things into the project store.
- No `.cake/` in the repo and the user wants project tracking → suggest
  `cake init`; don't run it unprompted.

## Item model

- An item **without status is a note**: plans, designs, decisions, context.
- An item **with status is a task**: `todo` | `doing` | `done` | `dropped`.
  `cake add` creates a note, `cake add --todo` a task, and `cake todo <id>`
  turns a note into a task.
- The first `# heading` of the body is the title. Keep it short.
- `parent` (one, `-p`), `tags` (`-t a,b`), `blocked_by` (`--blocked-by`),
  `due` (`--due 2026-10-30`), `priority` (`--prio 1..3`, 1 = high).
- Link other items in the body with `[[id]]`.
- `ready` = open task whose blockers are all closed. Notes are never ready.

## Commands

```sh
cake ready --json                       # open, unblocked tasks by priority/due: pick work here
cake ls --json [selectors]              # list; selectors below
cake show <id> --json                   # one item with body
cake add --json [--todo] [-p ID] [-t TAGS] [--blocked-by IDS] [--prio N] [--due D] -m TEXT
cake edit <id> --json -m -              # replace the body (frontmatter stays), from stdin
cake edit <id> --json --prio 1 --due 2026-10-30   # 0 / - clear them
cake todo|done|drop <ids> --json        # set status; done/drop set `closed`
cake tag <ids> a,~b --json              # add a, remove b
cake mv <id> <parent|-> --json          # re-parent / detach
cake block <id> --by <ids> --json       # <id> waits on <ids>; unblock removes
cake rm <ids> --json
```

Selectors (combine with and, `--or` for or): ids `a,b`, `-s TEXT` (title or
body), `--status todo,doing` (also `done`, `dropped`, `none` = notes), `-t tag,~not`,
`-p PARENT`, `--ready`, `--blocked`, `--by NAME`, `-r` (with children).

Multi-line content:

```sh
cake add --json --todo -p k3f9a2 -t cake -m - <<'EOF'
# Short title

Markdown body.
EOF
```

## Output

- Success: `{"v": 1, "items": [...]}` (items after the change), `rm` →
  `{"v": 1, "removed": [ids]}`. `ls` and `ready` leave out `body` unless
  `-l`; use `show <id>` for one item's body.
- Item: `id`, `title`, `body`, `status` (null = note), `tags`, `parent`,
  `children`, `blocked_by`, `blocks`, `ready`, `due`, `priority`, `created`,
  `closed`, `by`, `running`, `tracked_seconds`, `time`, `path`.
- Error: `{"v": 1, "error": {"code", "message"}}`, exit code 1.

| code | do |
|---|---|
| `not_found` | check the id with `cake ls --json -s …` |
| `invalid_input` | fix the arguments (message says what) |
| `conflict` | the store has an unresolved sync merge: report the files to the user and stop writing |
| `not_published` | only from sync; tell the user about `cake sync --publish` |
| `git`, `other` | report the message |

## Workflows

**Session start / "what next?"** — `cake ready --json`, propose from that
list (not from everything open). For background on a topic:
`cake ls --json -s <topic>` and `cake show <id> --json` on relevant notes.
Reuse existing items instead of adding duplicates.

**Plan non-trivial work** — write the plan as a note under the task it
belongs to: `cake add --json -p <task> -t <area> -m - <<'EOF' …`. Split
deliverables into `--todo` children of the task, one per deliverable.
Ordering constraints → `cake block <later> --by <earlier>`. Update your own
plan note with `cake edit <id> --json -m -` when it changes; for a plan
someone else wrote, add a child note instead.

**Found work mid-task** — `cake add --json --todo -p <current> -m "# …"` for
a subtask; unrelated → top level with a tag. Don't silently expand scope;
mention what you added.

**Finish** — before `cake done <id> --json`, read its `blocks`; afterwards
run `cake ready --json` and tell the user what became ready.

**Personal note** — `cake -g add --json -m "# …"`; find later with
`cake -g ls --json -s …`.
