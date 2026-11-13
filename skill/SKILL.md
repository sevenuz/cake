---
name: cake
description: Track tasks, plans, docs and ideas with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, ideas, questions, triage, time tracking or "remember this" (project or personal).
---

# cake

One markdown file per item in `.cake/items/` (project store, own git branch),
a separate store used from a project dir, or `~/.local/share/cake`
(personal store, `-g`). All known stores: `~/.config/cake/stores.json`
(name → store, dirs). The tree is on disk:
children of `sync.md` live in `sync/`. Every change through `cake` is
committed automatically. Your items get `by: <agent>`, changed items record
your session in `sessions`, your commits are tagged `[<agent>]`.

## Rules

- **Always `--json`.** stdout is one JSON document, errors included.
- **Never interactive:** content via `-m TEXT` or `-m -` with a heredoc. No
  `cake edit` without `-m`, no `cake config`.
- **Only through `cake`:** never edit `.cake/` files or run git in it.
- **Plans and docs go into the store**, never as files in the repo.
- **Edit only your own items** (`by` = you). To add to the user's items,
  create a child. Status and tags of the user's items only when asked
  (triage counts as asked: `mv` and tag, never the text).
- **Implement only approved items** (status `todo`/`doing`) that are yours
  or tagged `ai`.
- **Short but complete:** titles ≤ 60 chars, bodies as bullets, no prose.
  Aim for 3–5 bullets, but use more rather than lose details; split into
  children only when parts are worked on separately.
- **No `cake sync`** unless asked. Cite items by id.

## Names

The id is a slug of the title (first 4 words without filler, kebab-case),
and it is the file name. Ids never change.

- **Title starts with the topic:** `Merge driver for sync conflicts`, not
  `Implement a merge driver in cake` (→ `implement-merge-driver-cake`).
- **No tool, project or parent name** in the title or id: the directory
  already says where it belongs.
- **No status, dates or "plan/todo"** in the title: those change.
- Bad slug → pass an id: `cake add merge-driver …`, 2–4 words, kebab-case.
- Trees at most 3 levels; cross links with `[[id]]` (Obsidian style) or
  `--blocked-by` for order, never by nesting.

## Workflow

```
 user: cake add            inbox · (top level, no status, user's text)
                             │ you: triage
 you: cake add -p <idea>   planned · (cleaned up, split, `done when`)
      asks                 ○ #ask  blocks what it is about
                             │ user: cake done <ask> -m ANSWER (unblocks)
                             │ user: cake do <id>              (approve)
                           todo ○  (ready if no open blockers)
                             │ cake start <id>                 (timer runs)
                           doing ◐  stuck? new ask, next item
                             │ you: cake stop, cake tag <id> review,
                             │      list the commits in chat
                           review
                             ├ user "ok"       → you: cake done → ✓
                             ├ user "change X" → back to doing
                             └ user "drop"     → you: cake drop → ✗
```

- **Plans:** one note per idea or deliverable under the idea, bullets,
  `done when` included. Ordering with `--blocked-by`. Link with `[[id]]`.
- **Ready to approve** = `done when` present, no open asks, deadlines set
  when they matter. Only then is a planned item a candidate.
- **The user's side** is `cake` (= `cake morning`): paper todos, your asks
  by score, then your planned items to approve. Write asks and plans so
  they can be decided there without the chat.
- **Docs:** your reference notes are top-level and tagged `doc`.
- **Time:** start your timer on approved items you work on, stop it when you
  pause or request review. The owner of an item tracks its time.
- **Coming back:** when asked what to do next, use `cake ls --mine --json` and
  `cake ready --json`, propose what to discuss and what has priority.

## Triage

Inbox = the user's top-level items without status, children or tag
`triaged` (`cake ls --mine --status none --childless --json`, `parent`
null). Per item:

1. Sort: `cake mv` under a fitting existing item, or leave it top-level.
2. Clean up: your child with a clear title, bullets, `done when`. One
   thought with several deliverables → one child each.
3. Split when > 5 bullets or > 1 `done when`.
4. Ask what you cannot decide (below), at least the deadlines when the
   item sounds time-bound.
5. `cake tag <id> triaged`.

Personal store (`-g`): an idea that belongs to a project moves there first
(`cake -g mv <id> --to-store <name>`, names from stores.json), then triage
it in that store. Unsure which project → ask in `-g` with the store names
as options. Personal things stay in `-g`.

## Asks

A question for the user is an item: yours, `--todo`, tag `ask`, under the
item it is about, blocking it.

```sh
cake add --json --todo -p <item> -t ask -m - <<'EOF'
# Store answers in the ask or the item?

Context in 1–3 lines, only if the question is not clear without it.

- a) in the ask (recommended)
- b) in the item body
EOF
cake block <item> --by <ask> --json
```

- One question per ask, title is the question (≤ 60 chars, ends with `?`).
- Options a/b/c below, the recommended one first and marked.
- Deadline asks: `# Deadlines for <topic>?` with suggested dates
  (`due` = soft target, `deadline` = hard limit).
- The user answers with `cake done <ask> -m ANSWER`. Then apply it: edit
  your item, set `--due`/`--deadline`/`--prio` as answered.
- At most 5 new asks per run; ask the ones that unblock the most work.

## Agent run (routine, later a local model)

Per store in stores.json (`cake --store <path> … --json`), `-g` first so
moved ideas are triaged in their project:

1. `cake sync --json` (only in a routine).
2. Apply answered asks (`cake ls -t ask --status done --json`, newest).
3. Triage the inbox.
4. Work: `cake ready --by <you> -t ~ask --json` plus `-t ai`, highest
   score first. Code on a branch per item, never push to main. Stuck →
   ask, next item. Finished → stop timer, tag `review`.
5. `cake sync --json`, then a short summary: done, review, new asks.

## Commands

```sh
cake ls --json [selectors]              # tree: matches with children; -l adds bodies
cake ready --json                       # approved, unblocked, by score
cake show <id> --json                   # one item with body
cake add [ID] --json [--todo] [-p ID] [-t TAGS] [--blocked-by IDS] [--prio N] [--due D] [--deadline D] -m TEXT
cake edit <id> --json -m -              # replace the body (your items only)
cake edit <id> --json --prio 1 --due 2026-10-30 --deadline 2026-11-15   # 0 / - clear
cake start|stop <id> --json             # timer; start sets doing
cake do <ids> --json                    # approve: status todo (alias todo, reopen)
cake done|drop <ids> [-m NOTE] --json   # -m appends a note (answer, reason)
cake tag <ids> a,~b --json              # add a, remove b
cake mv <id> <parent|-> --json
cake mv <id> --to-store <name|global|path> --json   # with subtree, top-level there
cake block <id> --by <ids> --json       # unblock removes
cake rm <ids> --json
```

Ids: any unique prefix (≥ 2 chars) works; several ids space or comma
separated (`cake done a b`). `cake tag`: the last word are the tags. Selectors (and; `--or`): `-s TEXT`,
`--status todo,doing|done|dropped|none` (none = notes), `-t tag,~not`,
`-p PARENT`, `--mine`, `--by NAME`, `--ready`, `--blocked`, `--childless`,
`-A` (with done/dropped).

## Output

- `{"v": 1, "items": [...]}` (after the change), `rm` → `{"v": 1, "removed": [ids]}`.
  `ls`/`ready` without `body` unless `-l`.
- Item: `id`, `title`, `body`, `status` (null = note), `tags`, `parent`,
  `children`, `blocked_by`, `blocks`, `ready`, `score`, `due`, `deadline`,
  `priority`, `created`, `closed`, `by`, `sessions`, `running`,
  `tracked_seconds`, `time`, `path`.
- `score`: attention needed, higher first: deadline pressure (hard ≫ soft),
  10 per item waiting on it, priority, age.
- Error: `{"v": 1, "error": {"code", "message"}}`, exit 1. Codes:
  `not_found`, `ambiguous_id` (use a longer prefix), `invalid_input`,
  `conflict` (unresolved sync merge: tell the user, stop writing),
  `not_published`, `git`, `other`.

## Personal store

Things not about this project: `cake -g add --json -m "…"`, find with
`cake -g ls --json -s …`.
