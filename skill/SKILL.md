---
name: cake
description: Track tasks, plans, docs and ideas with the cake CLI (markdown items in a git-synced store). Use when the repo has a .cake/ directory, when starting non-trivial work there, or when the user mentions tasks, todos, plans, notes, ideas, questions, triage, time tracking or "remember this" (project or personal).
---

# cake

One markdown file per item in `.cake/items/` (project store, own git branch),
a separate store used from a project dir, or `~/.local/share/cake`
(personal store, `-g`). All known stores: `~/.config/cake/stores.json`.
A dir can use a store elsewhere (`cake --store <store> init [--parent
<goal>]` there); with a goal, `cake add` without `-p` lands under it
(`-p -`: top level), and `ls`, `ready` and the form show only that
goal's subtree (`ls --wide`: the whole store; `-p` picks another subtree).
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
  child or a comment (`cake edit <id> --comment …`), which lands in the log.
- **Work only on ready tasks assigned to you or to `ai`** (`cake ready`
  shows exactly those). Never sort the inbox: that is the user's.
- **Readable without the chat:** the user decides in `cake`, days later.
  Bodies follow the layout below: as short as possible while complete. No
  internal shorthand, no bare file or function names without saying what
  they do. Titles ≤ 60 chars.
- **Designs agreed in chat** go into a `doc` note under the plan before any
  implementation, complete enough to build from without the chat:
  decisions with reasons, rejected options, exact formats and keys, edge
  cases, an example. A big design: one child note per topic (below).
- **No `cake sync`** unless asked or in an agent run. Cite items by id.
- **Capture from the chat:** when the user mentions something for later
  ("todo", "later", "idea", "we should …") or a decision worth keeping,
  offer `cake add` in one line. Once per topic, never when they said no.

## Item layout

Short full sentences, bullets only for real lists. Fields in this order:

```
# Merge driver for sync conflicts

Context: two clones editing one item end in a git conflict cake cannot read.
Why: sync stops and the user has to fix markdown by hand.
Change: a git merge driver that merges front matter per key, bodies per line.
Done when: the two-clone test syncs without a conflict marker.
Not: rebasing the store branch (rewrites history others pulled).

## Log
- 2026-11-18 changed: driver in sync/merge; try: cake sync in both clones
- 2026-11-19 comment (sevenuz): also for archived items
```

- `Context:` 1–3 sentences, readable cold; `Why:` the problem in plain
  words; `Change:` what gets done; `Done when:` a checkable state; `Not:`
  rejected options or out of scope (optional).
- Notes (docs, rules, goals) use the fields that fit; docs add `Decided:`,
  formats, edge cases, examples.
- **Above `## Log` is the current state.** On your own items rewrite it
  (`cake edit <id> -m`) instead of appending; history goes to `## Log` as
  one dated line per event. cake writes comments, answers, review notes and
  `discuss` lines there itself.
- The user's items: body untouched; only log lines and children.

## Names

The id is the file name. Without an id `cake add` takes a random one
(`kqejfn`). Ids change only through `cake rename <id> <new>` (links,
blockers and children follow).

- Title starts with the topic: `Merge driver for sync conflicts`.
- No tool, project or parent name, no status or dates in the title.
- Always pass an id: topic first, 1–3 words, easy to type (`cake add
  sync-merge …`). Goals and plans get one word (`dashboard`, `form`,
  `sync`) so the user can file with `-p dashboard`; children start with
  the parent's topic (`dash-period`, `dash-pie`).
- Rename only random ids (exactly 6 letters, no dash) in triage, to a topic
  id, listed in the summary. Every other id only when the user asks: an id
  the user typed is theirs.

## Tree

No depth limit; nest by meaning.

- A child belongs to exactly one parent and is part of it: the parent is
  finished only when the child is. What touches several parents gets a
  `[[id]]` link or `--blocked-by`, not a place in the tree.
- Goals are notes tagged `goal` and may nest (`phd` › `gekkofs` ›
  `gekkofs-paper`); rules and goals apply down the tree.
- Tasks split as deep as needed; a task with work children is a plan, only
  leaves are worked.
- Docs go under their plan. A big design: the `doc` note holds a short
  summary, each topic is an untagged child note; children link each other
  with `[[id]]`. No index, the tree shows them.
- **Handoff:** one living note per goal, `<goal>-handoff`, tagged
  `handoff`: 2–3 sentences of current state and next step, rewritten, never
  a second note. Its children are the open threads (half-made decisions,
  context no item holds). A thread that gets concrete graduates: `cake edit
  <id> --parent <goal or plan>` and make it a task or doc. A settled thread
  is closed or merged into a doc.

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
   --json`. A new `comment (…)` line in `## Log` (older items: at the end
   of the body) is a change request; a `blocked_by` item that is now
   `done` carries the user's answer there. Rework accordingly and rewrite
   the state above the log. Items tagged `discuss` (assigned to the user,
   a `discuss (…): <doubts>` log line) were sent back from review as not
   ok yet: the doubts are in that line. Bring them up first in the next
   chat, never rework them unasked; after the talk either the user closes them, or you
   rework and remove the tag (`cake edit <id> --tag ~discuss --assignee
   <you>`).
   Dropped items with `commits` and without the tag `reverted`: the
   user threw the work away. Add a task under the same parent that
   reverts those commits (assignee you, owner the user, `--blocked-by`
   nothing), then `cake edit <dropped> --tag reverted`.
4. Garden (triage and agent runs): keep the tree filed and lean, per goal,
   deepest goals first (in a scoped dir only that goal's subtree).
   - top-level items that are not goal notes: move under the matching
     goal; direct children of a goal that are not goals, plans, docs or
     handoffs: move into the best fitting subgoal or plan. `cake edit <id>
     --parent <x>`, only the parent changes, never the user's text; every
     move in the summary; nothing fits → it stays, name it;
   - random ids (6 letters, no dash): `cake rename` to a topic id, in the
     summary;
   - open items that overlap or repeat each other: never merge on your
     own, ask "Merge <a> into <b>?" (a: merge, b: keep both, c: drop one);
   - ready tasks without `Why`/`Done when` that are not yours to plan
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
   - otherwise `cake start <id>`, do the work (git as the repo's own
     rules say, e.g. AGENTS.md; without any: commit on the current branch,
     push only when asked), then
     `cake review <id> --commit <hashes> -m "changed: …; try: …" --json`.
6. Your own findings and docs: `cake add --note -t doc …`, never the inbox.
7. Update the handoff of each goal you worked in (Tree, above).
8. `cake sync --json`, then a short summary: reviews, questions, blocked,
   moves and renames, what you found to merge.

## Chat mode

While the user is in the chat, the chat is the work loop and cake the
record. Nothing is asked or answered twice.

- **Session start:** read the handoff of the goal you work in
  (`<goal>-handoff` and its children); bring up `discuss` items first.
- **Session end** (the user leaves, or a big topic is done): rewrite that
  handoff: state and next step, open threads as children, graduated
  threads moved out. Never a new handoff note.
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
          [--set key=value] [--add key=value] [--unset key]   # free header fields
# store config .cake/config.hjson `fields`: types (number, list, date, duration, text)
# and derived ones ({ wh: { derive: "tokens_in * 0.00025" } }), shown under `fields` in json
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
`--where key=value`, `--has key` (free fields),
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
