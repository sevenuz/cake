---
name: cake
description: Track tasks, plans and notes with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, time tracking or "remember this" (project or personal).
---

# cake

cake stores tasks and notes as one markdown file per item in `.cake/items/`
(project store, its own git branch) or `~/.local/share/cake` (personal global
store, `-g`). Every change through `cake` is committed to the store
automatically. Items you create get `by: <agent>` (from `AI_AGENT`), items
you change record your session id in `sessions`, and your commits are tagged
`[<agent>]`, all without you doing anything.

## Rules

- **Always pass `--json`.** stdout is one JSON document, errors included.
- **Never interactive:** always give content with `-m TEXT` or `-m -` plus a
  heredoc. Never call `cake edit` without `-m`, or `cake config`.
- **Never touch `.cake/` directly:** no editing `.cake/items/*.md`, no git
  inside `.cake/`. Everything through `cake`, so commits stay consistent.
- **No loose plan files:** when a store exists, don't create `PLAN.md`,
  `TODO.md` or similar in the repo. Put them into the store.
- **Edit only your own items** (`by` is your agent name, e.g. `claude-code`).
  Never change body, priority, due, status or tags of items others created,
  unless the user asks. To add to someone else's item, create a child:
  `cake add -p <id> …`.
- **Track your time:** `cake start <id>` when you begin work on your task,
  `cake stop <id>` when you pause or finish. Only on your own items; the
  owner of an item tracks its time.
- **Never close your tasks:** when finished, stop the timer and tag `review`.
  The user closes them.
- **Short:** titles ≤ 60 chars; bodies ≤ 5 bullets, no prose. Bigger → split
  into child items.
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
cake start|stop <id> --json            # your timer; start sets doing
cake todo|done|drop <ids> --json        # set status; done/drop set `closed`
cake tag <ids> a,~b --json              # add a, remove b
cake mv <id> <parent|-> --json          # re-parent / detach
cake block <id> --by <ids> --json       # <id> waits on <ids>; unblock removes
cake rm <ids> --json
```

Selectors (combine with and, `--or` for or): ids `a,b`, `-s TEXT` (title or
body), `--status todo,doing` (also `done`, `dropped`, `none` = notes), `-t tag,~not`,
`-p PARENT`, `--ready`, `--blocked`, `--by NAME`, `--childless`, `-r` (with children).

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
  `closed`, `by`, `sessions`, `running`, `tracked_seconds`, `time`, `path`.
- Error: `{"v": 1, "error": {"code", "message"}}`, exit code 1.

| code | do |
|---|---|
| `not_found` | check the id with `cake ls --json -s …` |
| `invalid_input` | fix the arguments (message says what) |
| `conflict` | the store has an unresolved sync merge: report the files to the user and stop writing |
| `not_published` | only from sync; tell the user about `cake sync --publish` |
| `git`, `other` | report the message |

## Workflow with the user

Tags: one area tag (`git`, `tui`, …) plus `ask` and `review`.

```
idea      (user,  note)   stays as the user wrote it
├─ plan   (agent, note)   decisions, ≤ 5 bullets
├─ task   (agent, todo)   one deliverable
│  └─ sub (agent, todo)   divide and conquer
├─ task   (agent, todo)   --blocked-by the first
└─ Q      (agent, note, tag ask)
```

1. **New ideas** — `cake ls --json --status none --childless`; the user's
   notes among them (`by` is not you) are not worked out yet.
2. **Discuss** in chat. Propose the breakdown as bullets; write nothing
   until the user agrees.
3. **Write down** — under the idea: one plan note, then one `--todo` per
   deliverable. Split big tasks into children; order with `--blocked-by`.
   Priority only when the user names one.
4. **Work** — pick from `cake ready --json`. `cake start <id>`; write each
   task once, show progress with status and tags. New findings → child
   items. A decision needed → child note tagged `ask`; remove the tag when
   answered.
5. **Finish** — `cake stop <id>`, `cake tag <id> review`; tell the user what
   is ready for review and what became unblocked (`blocks`, `cake ready`).
   The user closes with `cake done`.

**Context** — `cake ls --json -s <topic>`, `cake show <id> --json` on
relevant notes. Reuse items instead of adding duplicates.

**Sessions** — `sessions` lists the agent sessions that changed an item;
resume one to see how a task was done (Claude Code: `claude --resume <id>`).

**Personal note** — `cake -g add --json -m "# …"`; find later with
`cake -g ls --json -s …`.
