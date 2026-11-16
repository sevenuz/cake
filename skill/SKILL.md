---
name: cake
description: Track tasks, plans, docs and ideas with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, ideas, questions, triage, time tracking or "remember this" (project or personal).
---

# cake

One markdown file per item in `.cake/items/` (project store, own git branch),
a separate store used from a project dir, or `~/.local/share/cake`
(personal store, `-g`). All known stores: `~/.config/cake/stores.json`.
The tree is on disk: children of `sync.md` live in `sync/`. Every change
through `cake` is committed automatically. Your items get `by: <agent>`,
changed items record your session in `sessions`, commits are tagged
`[<agent>]`.

## Rules

- **Always `--json`.** stdout is one JSON document, errors included.
- **Never interactive:** content via `-m TEXT` or `-m -` with a heredoc. No
  `cake edit` without `-m`, no `cake config`, no `cake` / `cake form`.
- **Only through `cake`:** never edit `.cake/` files or run git in it.
- **Plans and docs go into the store**, never as files in the repo.
- **The user's text is theirs:** never edit the body of their items; add a
  child or a comment (`cake edit <id> --comment …`).
- **Work only on ready tasks assigned to you or to `ai`** (`cake ready`
  shows exactly those). Never sort the inbox: that is the user's.
- **Short but complete:** titles ≤ 60 chars, bodies as bullets, no prose.
  Aim for 3–5 bullets, more rather than losing details.
- **Designs agreed in chat** go into a `doc` note under the plan before any
  implementation, complete enough to build from without the chat:
  decisions with reasons, rejected options, exact formats and keys, edge
  cases, an example. Bullet limits apply to plans, not to these.
- **Readable without the chat:** the user decides in `cake`, days later.
  First bullet `why:` the problem in plain words, then what changes, then
  `done when`. No internal shorthand, no bare file or function names
  without saying what they do.
- **No `cake sync`** unless asked or in an agent run. Cite items by id.
- **Capture from the chat:** when the user mentions something for later
  ("todo", "later", "idea", "we should …") or a decision worth keeping,
  offer `cake add` in one line. Once per topic, never when they said no.

## Names

The id is a slug of the title (first 4 words without filler) and the file
name. Ids change only through `cake rename <id> <new>` (links, blockers and
children follow); rename only when the user asks.

- Title starts with the topic: `Merge driver for sync conflicts`.
- No tool, project or parent name, no status or dates in the title.
- Always pass an id, the slug is rarely good: topic first, 1–3 words,
  easy to type (`cake add sync-merge …`). Goals and plans get one word
  (`dashboard`, `form`, `sync`) so the user can file with `-p dashboard`;
  children start with the parent's topic (`dash-period`, `dash-pie`).
- Trees at most 3 levels; cross links with `[[id]]` or `--blocked-by`.

## States

```
 cake add ──► inbox ──(user sorts in `cake`)──► note | task | dropped
                                                     │ assignee, owner, due, deadline, prio
                         task ──start──► doing ──review──► review ──► done
                           ▲                                 │ (owner checks)
                           └──── comment (change request) ◄──┘
```

| status | meaning |
|---|---|
| `inbox` | new, not sorted; only the user sorts it |
| (none) | a note: knowledge, idea, goal, rule; nothing to do |
| `task` | to do; `assignee` does it, `owner` reviews it |
| `doing` | in progress, timer runs |
| `review` | finished; the owner checks it |
| `done`, `dropped` | closed |

- **Ready** = task (or doing) without open blockers and without open work
  children. A parent waits for its children and goes to `review` by itself
  when all are closed.
- **Questions are tasks:** assigned to the user, blocking your task, options
  in the body. The user's answer closes it ("answer: …" appended).
- **Comments** (the user's change requests, remarks) are appended to the
  item's body as `comment (<who>, <date>): …`; the item is back at `task`.
- **Tags are labels only** (`doc`, `rule`, `goal`, `cli`, …), no state.
- **Brake:** while `max_review` items (store config, default 8, -1 off)
  wait in review, `cake ready` gives you nothing and `cake start` fails
  with `waiting_for_review`. Stop and summarize.

## Agent run

Per store (`cake --store <path> … --json`), when asked to triage or work:

1. `cake sync --json` (only in a routine or when asked).
2. Read rules: notes tagged `rule`/`goal` up the tree of what you touch
   (top level: the whole store). They override defaults here.
3. Answers and comments: `cake ls -l --assignee <you>,ai --status task
   --json`. A new `comment (…)` line at the end of a body is a change
   request; a `blocked_by` item that is now `done` carries the user's
   answer at the end of its body. Rework accordingly. Items tagged
   `discuss` (assigned to the user, a `discuss (…): <doubts>` line at the
   end) were sent back from review as not ok yet: the work so far stays
   as described above that line, the doubts follow it. Bring them up first in the next chat, never rework
   them unasked; after the talk either the user closes them, or you
   rework and remove the tag (`cake edit <id> --tag ~discuss --assignee
   <you>`).
4. Garden (triage and agent runs): keep the tree filed and lean.
   - top-level items that are not goal notes (sorted feedback, loose
     tasks): move under the matching goal, `cake edit <id> --parent
     <goal>`; only the parent changes, never the user's text; no goal
     fits → name it in the summary;
   - open items that overlap or repeat each other: never merge on your
     own, ask "Merge <a> into <b>?" (a: merge, b: keep both, c: drop one);
   - ready tasks without `why`/`done when` that are not yours to plan
     (old ideas, vague dumps): ask "Plan or park <id>?" (park = `cake
     note`, `cake do` brings it back);
   - a parent that only collects leftovers (many done children, few open):
     propose a small `<topic>-polish` task for the rest so it can close.
5. Loop until `cake ready --json` is empty:
   - take the highest `score`, `cake show <id> --json`;
   - a task without children that is too big → split: subtasks with
     `cake add --task -p <id> --assignee <you> --est 1h -m …` (they
     inherit the owner), order with `--blocked-by` between siblings, never
     the parent; `--est` is your honest guess of the work (2h30m, 45m), it
     feeds the critical path to a goal's deadline;
   - a decision that is hard to undo or changes the scope → ask (below),
     then continue with the next ready item;
   - otherwise `cake start <id>`, do the work (code on a branch per item,
     never push to main), then
     `cake review <id> --commit <hashes> -m "changed: …; try: …" --json`.
6. Your own findings and docs: `cake add --note -t doc …`, never the inbox.
7. `cake sync --json`, then a short summary: reviews, questions, blocked,
   what you filed or found to merge.

## Chat mode

While the user is in the chat, the chat is the work loop and cake the
record. Nothing is asked or answered twice.

- **Ask in chat**, not as question items. Write the answer into cake
  yourself: into the task, a design `doc` note, or a question item created
  and closed at once (`cake done <id> -m "answer: …"`) when it is worth
  keeping as its own item.
- **"go" starts the work** right away in this session: `cake start`, work,
  `cake review`. No separate agent run, no "now run the agent".
- **Review both ways:** "ok <id>" in chat → `cake done <id>`; a change
  request in chat → `cake edit <id> --comment …`, then rework. The form
  stays for what the user checks later.
- **Question items in cake** only when the user is away (agent run,
  triage) or says "decide later" / "ask me in cake".
- Ids from cake in your answers, so the user can find the items.

## Questions

When the user is away (see Chat mode):

```sh
cake add --json --task -p <task> --assignee me -m - <<'EOF'
# Store answers in the task or a note?

Context in 1–3 lines, only if needed.

- a) in the task (recommended)
- b) in a note
EOF
cake edit <task> --blocked-by <question> --json
```

- One question per item, title is the question (≤ 60 chars, ends with `?`).
- Options a/b/c (at most 4), the recommended one first and marked.
- Date questions: `# Deadline for <topic>?` with suggested dates and why.

## Commands

```sh
cake ls --json [selectors]              # tree: matches with children; -l adds bodies
cake ready --json                       # = ls --ready: ready tasks (agents: yours and ai)
cake show <id> --json                   # one item with body
cake add [ID] --json [--task|--note] [-p ID] [--assignee WHO] [-t TAGS] [--blocked-by IDS] [--prio N] [--due D] [--deadline D] [--est 2h] -m TEXT
cake set <status> <ids> [-m TEXT] [--commit H1,H2] --json   # inbox note task doing review done dropped; -m appends
# aliases: done, drop, do (task), note, review: `cake review <id> -m "changed …"`
cake edit <ids> --json [-m BODY] [--prio N] [--due D] [--deadline D] [--est 2h|-] [--commit H1,H2] [--assignee WHO] [--owner WHO]
          [--tag a,~b] [--blocked-by x,~y] [--parent ID|-] [--comment TEXT]
# -m replaces the body (your items only); --comment appends and gives it back to the assignee
# dates: 2026-11-30, 30.11., today, tomorrow, mon..sun, +3d, 2w; stored as YYYY-MM-DD
# who: me (the user), ai (any agent), an agent or person name, - nobody
cake start|stop <id> --json             # timer; start sets doing
cake mv <id> --to-store <name|global|path> --json   # with subtree, top-level there
cake rename <id> <new> --json           # new id; children, blockers, [[links]] follow
cake rm <ids> --json
cake archive [--older 3m] [-n] --json   # -n: only list; closed subtrees → items/archive/ (only when asked)
```

Ids: any unique prefix (≥ 2 chars); several ids space or comma separated.
Selectors (and; `--or`): `-s TEXT`,
`--status inbox,note,task,doing,review,done,dropped`, `-t tag,~not`,
`-p PARENT`, `--assignee WHO`, `--mine`, `--by NAME`, `--ready`,
`--blocked`, `--childless`, `-A` (with done/dropped), `--archive` (also
archived items; `show` finds them by id anyway).

## Output

- `{"v": 1, "items": [...]}` (after the change), `rm` → `{"v": 1, "removed": [ids]}`.
- `ls`/`ready` are short: `id`, `title`, `status` (null = note), `tags`,
  `parent`, `children`, `blocked_by`, `blocks`, `ready`, `score`, `due`,
  `deadline`, `priority`, `running`, `by`, `assignee`, `owner`. Use `-l` or
  `show` for the rest: `body`, `created`, `closed`, `sessions`, `time`,
  `tracked_seconds`, `usage`, `footprint`, `path`.
- List short to choose, then read what you work on: `cake show <id>` before
  editing, answering, implementing or reviewing. Every listed item costs
  tokens; prefer narrow selectors.
- Error: `{"v": 1, "error": {"code", "message"}}`, exit 1. Codes:
  `not_found`, `ambiguous_id`, `invalid_input`, `waiting_for_review` (stop
  working, summarize), `conflict` (unresolved sync merge: tell the user,
  stop writing), `not_published`, `git`, `other`.

## Personal store

Things not about this project: `cake -g add --json -m "…"`. Ideas in `-g`
that belong to a project are moved by the user while sorting.
