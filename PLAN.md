# cake — rewrite plan

Goal: one CLI that is task tracker, time tracker, todo list and a queryable
markdown knowledge store (also for AI agents), synced through git without
touching the user's working tree.

Stages:

1. **Stage 1** — new storage format, git worktree sync, `--json`, htop-like TUI
   dashboard on bare `cake`, Claude skill.
2. **Stage 2** — `cake mcp` (MCP stdio server).
3. **Stage 3** — SQLite index + FTS5 full-text search.

No migration from `cake.md` / `cake.json`. The single-file formats, `-i/-o`
and the json store are deleted.

---

## 1. Data model

### Directory layout

```
.cake/                       # git worktree of orphan branch `cake` (or plain dir)
  .gitignore                 # index.sqlite, .obsidian/workspace*.json
  .gitattributes             # later: items/*.md merge=cake
  items/
    k3f9a2.md
    cake.md                  # custom ids allowed: [a-z0-9-]+
```

One file per item. Two devices editing different items never conflict.
`.cake/` can be opened directly as an Obsidian vault.

### Item file

```markdown
---
id: k3f9a2
status: doing                # todo | doing | done | dropped; absent = note
tags: [cake, storage]
parent: "[[p0x7qe]]"         # single parent, Obsidian wikilink, optional
blocked_by: ["[[m2k8zz]]"]   # optional, items that must close first
aliases: ["File per item"]   # = title, makes Obsidian link autocomplete work
created: 2026-10-22T09:12:00+02:00
by: sevenuz                  # creator, set once: CAKE_AUTHOR | agent | git user.name
closed: 2026-10-23T18:00:00+02:00   # set when status -> done|dropped
due: 2026-10-30              # optional, date or datetime
priority: 2                  # optional, 1 (high) .. 3 (low)
time:
  - 2026-10-22T09:15:00+02:00/2026-10-22T10:40:00+02:00
  - 2026-10-22T14:00:00+02:00/          # open interval = running timer
---
# File per item

Body is free markdown. First `# heading` is the title.
```

Rules:

- **Only the child stores the relation.** `children` is computed on load
  (`HashMap<parent, Vec<child>>`). Re-parenting touches exactly one file.
- **Dependencies (Beads-style), same rule:** only the waiting item stores
  `blocked_by`; `blocks` is computed on load. Other links are `[[id]]`
  wikilinks in the body → Obsidian graph shows them.
- **Notes vs tasks:** no `kind` field. An item without `status` is a note
  (never in `ready`); any status command (`todo`, `start`, `done`, `drop`)
  turns it into a task. `cake add` creates a note, `--todo` a task.
- **Author:** `by` is written once on creation, never changed (no
  conflicts). Commits by an agent end with `[agent]`; `git log` shows who
  changed what.
- **Ready is computed, never stored:** ready = status todo|doing and every
  `blocked_by` item is done|dropped; blocked = at least one open blocker.
  Storing it would rewrite files when a blocker closes → conflicts.
- **No stored `last_modified`.** It changes on every edit and guarantees
  conflicts. Use file mtime for "recently touched" sorting; `git log` when
  exact history is needed.
- **Times:** RFC 3339 with local offset, second precision. Intervals in ISO
  8601 `start/end` notation; empty end = running. At most one open interval
  per item; starting a new timer on another item stops the running one
  (configurable `single_timer: true`).
- **IDs:** 6 chars from `0123456789abcdefghjkmnpqrstvwxyz` (~10⁹ space, safe
  for offline multi-device). CLI accepts any unique prefix (min 3), like git.
  Display uses shortest unique prefix. Custom ids allowed.
- **Deterministic serialization:** fixed key order, empty/absent optional
  fields omitted, lists inline for `tags`, block for `time`. Minimal diffs.
- **Atomic writes:** write `items/.k3f9a2.md.tmp`, then `rename`.
- Unknown frontmatter keys are preserved (round-trip), so Obsidian plugins or
  the user can add fields.

### Status semantics

| action             | effect                                                     |
|--------------------|------------------------------------------------------------|
| `cake add`         | no status (note); `--todo` → `status: todo`                |
| `cake start <id>`  | open interval, `status: doing`, stop other running timer   |
| `cake stop [id]`   | close interval (no id = the running one), status unchanged |
| `cake done <id>`   | close interval if open, `status: done`, set `closed`       |
| `cake drop <id>`   | `status: dropped`, set `closed`                            |
| `cake todo <id>`   | `status: todo`, remove `closed` (alias `reopen`)           |
| `cake block <id> --by <ids>`   | add `blocked_by` edges (cycle check)           |
| `cake unblock <id> --by <ids>` | remove `blocked_by` edges                      |

### Rust structure

```rust
pub struct Item {
    pub id: String,
    pub status: Option<Status>,  // Todo | Doing | Done | Dropped, None = note
    pub tags: Vec<String>,
    pub parent: Option<String>,
    pub blocked_by: Vec<String>,
    pub created: DateTime<FixedOffset>,
    pub by: Option<String>,
    pub closed: Option<DateTime<FixedOffset>>,
    pub due: Option<Due>,
    pub priority: Option<u8>,
    pub time: Vec<Interval>,     // Interval { start, end: Option<_> }
    pub extra: serde_yaml::Mapping, // unknown keys, round-tripped
    pub body: String,
}

pub struct Store {
    root: PathBuf,
    items: HashMap<String, Item>,
    children: HashMap<String, Vec<String>>, // derived
    blocks: HashMap<String, Vec<String>>,   // derived from blocked_by
    dirty: HashSet<String>,                 // only these get written
}
```

`Store::save()` writes only dirty items, returns changed paths (for git add
and for the commit message).

YAML crate: `serde_yaml` is unmaintained; pick a maintained fork at
implementation time (`serde_yaml_ng` / `serde_norway`) — verify status first.
Writer is hand-rolled for deterministic layout; parser uses the crate.

---

## 2. Store discovery

Order:

1. `--store <path>` flag / `CAKE_STORE` env.
2. Walk up from `$PWD`: first dir containing `.cake/items/` → project store.
3. Inside a git repo whose remote has `cake/main` but no `.cake/`
   worktree yet → hint to run `cake init` (attach, see §3).
4. Fallback: global store `$XDG_DATA_HOME/cake` (`~/.local/share/cake`),
   itself a plain git repo on `main` (optional remote).

`cake -g ...` forces the global store from anywhere, also inside a project. Config keeps a list of known stores
(appended on init) for cross-store stats later.

---

## 3. Git: worktree-based sync

Principle: a small, solid base. One branch, one worktree, one commit per
command, explicit sync. Everything else (personal branches, merge driver,
auto sync) is added on top later without changing the base.

### What the current code does (and why it goes)

Every command: `git stash` → `git checkout cake` → (fetch + rebase) → edit
`cake.md` → `git add` + `commit` → (push) → `git checkout <orig>` →
`git stash pop`.

Problems:

- It rewrites the user's working tree twice per command. Editors reload,
  `cargo`/build tools see changed mtimes and rebuild.
- `git stash` grabs unrelated uncommitted work; untracked files aren't
  stashed and can block the checkout.
- A crash or SIGKILL between stash and pop strands the user on the cake
  branch with work stashed (the Drop guard helps for errors, not kills).
- The `cake` branch was created from HEAD, so it carries the whole code
  history; `cake.md` also exists on `main` (the `M cake.md` in git status).
- `rebase` per command replays many tiny commits → repeated conflicts.

### What a worktree is

A git repo normally has exactly one working directory. `git worktree add`
attaches **additional** working directories to the same repository:

```
repo/
  .git/                       # the one real repository
    objects/  refs/           # shared by all worktrees
    worktrees/.cake/          # per-worktree state for .cake/
      HEAD                    # -> ref: refs/heads/cake/main
      index                   # its own staging area
      gitdir                  # back-pointer to repo/.cake/.git
  src/ ...                    # main worktree, on `main`
  .cake/                      # linked worktree, on `cake/main`
    .git                      # a FILE: "gitdir: repo/.git/worktrees/.cake"
    items/...
```

Properties:

- Each worktree has its own `HEAD`, index and files. Committing in `.cake/`
  never touches the main worktree's files or index — no stash, no checkout.
- Objects and refs are shared: a commit in `.cake/` immediately moves
  `refs/heads/cake/main` for the whole repo; pushing works from either.
- A branch can be checked out in only one worktree per clone. `git checkout
  cake/main` in the main worktree is refused — a safety feature here. Each
  device has its own clone and its own `.cake/`; devices meet via the remote.
- A worktree needs a ref, otherwise its commits are unreachable and get
  garbage-collected. We use a normal branch (hidden refs like
  `refs/cake/*` stay a future option for "pushed but invisible").
- Worktree metadata (`.git/worktrees/`) is local, **not cloned**. On a fresh
  clone cake re-attaches (below). `git worktree prune` / `repair` clean up
  if `.cake/` is deleted or the repo is moved.

### Branch naming

- `cake/main` — the (only, for now) project store branch. Orphan: no shared
  history with code, contains only `items/`, `.gitignore`, `.gitattributes`.
- Future: personal branches `cake/<git user.name slug>` with their own
  worktree. Everything lives under `cake/`, because git cannot have a branch
  `cake` and `cake/x` at the same time (refs are paths).
- Branch name is a config value (`git_branch: "cake/main"`), not hard-coded.

### `cake init` (in a git repo)

```sh
# A) remote already has cake/main (second device / fresh clone): attach
git fetch origin cake/main
git worktree add .cake cake/main        # DWIM: local branch tracking origin

# B) otherwise: create orphan branch + worktree (git >= 2.42)
git worktree add --orphan -b cake/main .cake
mkdir .cake/items && write .cake/.gitignore
git -C .cake add -A && git -C .cake commit -m "cake: init"

# both: hide .cake/ from the main worktree without committing to main
echo "/.cake/" >> "$(git rev-parse --git-common-dir)/info/exclude"
```

Not a git repo, or `disable_git: true` → plain `.cake/` directory, every git
step skipped.

When a repo has `origin/cake/main` but no `.cake/`, cake prints a hint to run
`cake init`.

### Global store

`~/.local/share/cake` (XDG data dir): a plain git repo on branch `main`, no
worktree. `cake -g <cmd>` uses it from anywhere, also inside a project. Same
commit/sync code; only repo dir and branch differ. Remote optional:
`cake -g init`, later `cake -g sync --publish <url>`.

### Per-command commit (local, offline, fast)

After every mutating command:

```sh
git -C .cake add -- items/<changed>.md ...
git -C .cake commit -q -m "cake start k3f9a2"   # skipped if nothing staged
```

- Always `git -C <store>`; never depend on `$PWD`.
- Shell out to the `git` binary (user's credentials, ssh agent, config,
  hooks just work). Every call checks exit status and surfaces stderr.
- A lock file (`.cake/.lock`, `fd-lock`) serialises concurrent cake
  processes (TUI + CLI in another shell).

### Sync (`cake sync`, manual only)

Runs only when invoked: `cake sync` or `S` in the TUI. Never implicitly.

```sh
git -C .cake fetch origin cake/main
git -C .cake merge --no-edit origin/cake/main    # merge, not rebase
git -C .cake push origin cake/main
```

- **Merge, not rebase:** one commit per command → rebase would replay many
  commits and can hit the same conflict repeatedly. Merge resolves once.
- **No upstream set** → error: "store is local-only; run `cake sync
  --publish [remote]`". `--publish` = `git push -u <remote> cake/main`, once.
  Publishing makes data visible to everyone with repo access, so it is
  always explicit.

### Conflicts

Same item edited on two devices between syncs:

- The merge is left in progress; conflicted `items/*.md` contain standard
  conflict markers.
- Before every command cake checks for `MERGE_HEAD` in the store. While
  present: **reads work**, **writes are refused** listing the conflicted
  files.
- User resolves in `$EDITOR` / Obsidian, then `cake sync --continue`
  (verifies no markers left, commits, pushes) or `cake sync --abort`
  (`git merge --abort`).

Later (on top, no base change): `.gitattributes` `items/*.md merge=cake` +
local `git config merge.cake.driver "cake merge-driver %O %A %B"` for
field-wise 3-way merge (union of `time`, `tags`, `blocked_by`, …).

### Phone / foreign editors

cake must read files it didn't write (Obsidian, phone git apps, Termux):

- **Lenient parsing:** missing frontmatter or fields → defaults (no status
  = note, `created` = file mtime); `id` = frontmatter id or
  filename stem (`buy milk.md` → id `buy milk`, quoted when needed);
  unknown keys preserved.
- Files are not rewritten on read. The first edit through cake normalises
  the file.
- Easiest phone target: the global store (small dedicated repo on `main`)
  via Obsidian mobile + Obsidian Git, Working Copy (iOS) or GitJournal /
  Termux (Android). Project stores on an orphan branch inside a code repo
  are possible but fiddlier for phone apps.

---

## 4. CLI

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

- stdout = exactly one JSON document; nothing else (no color, no hints).
- `ls` → array of items; `show` → item; mutating commands → the changed
  item(s) after the change; `stats` / bare `cake --json` → stats object;
  `sync` → `{ "merged": n, "pushed": bool, "conflicts": [...] }`;
  writes during an unresolved merge → error code `conflict`; sync without
  upstream → `not_published`.
- Errors → `{"error": {"code": "not_found", "message": "…"}}` on stdout,
  exit code ≠ 0. Stable codes: `not_found`, `ambiguous_id`, `conflict`,
  `invalid_input`, `git`.
- Item JSON = frontmatter fields + `title`, `body`, `children` (derived),
  `blocks` (derived), `ready` (bool), `tracked_seconds`, `running` (bool),
  `path`.
- Top-level `"v": 1` for schema versioning.

---

## 5. TUI dashboard (bare `cake`)

Crates: `ratatui` + `crossterm`. Alternate screen + raw mode → `q` restores
the previous terminal content, nothing left behind. Restore also from a
panic hook and on SIGTERM. (termimad stays for `show` / `ls -l`; align its
crossterm version with ratatui's to avoid two copies.)

Not a tty (pipe, script) → print static one-shot summary instead.

```
┌ cake · ~/git/cake/.cake ──────────────────────────── synced 4m ago ┐
│ ▶ k3f9a2 File per item                     01:25:13   [s]top        │
│ today  3h40 ████████████░░░░░  week 18h05 ██████████████░░░  /25h   │
│ 14d ▂▅▇▃▁ ▆█▅▄▂ ▃▇   streak 6d   done 7d 11   overdue 1             │
├ Tasks ─ Notes ─ Time ─ Tags ──────────────────────────── /status:todo ┤
│ ID     ST  PRIO TITLE                         TAGS        TIME   DUE │
│ k3f9a2 ▶    1   File per item                 cake        3h05   30.10│
│ p0x7qe ○    2   ├ Git worktree sync           cake,git    1h10       │
│ m2k8zz ○    -   └ TUI dashboard               cake           -       │
│ a81hfq ✓    -   Paper draft                   gekko       9h40       │
│ …                                                                    │
├──────────────────────────────────────────────────────────────────────┤
│ preview: body of selected item (markdown, scrollable)                │
└ q quit  j/k move  ⏎ open  s start/stop  d done  a add  e edit  / filter  t tree  S sync ┘
```

Panels:

- **Header:** running timer (ticks every second), today/week meters vs
  configurable weekly goal, 14-day sparkline, streak, done-in-7-days,
  overdue count, sync status (ahead/behind from last fetch, unpublished,
  pending conflicts). No automatic network access; `S` runs `cake sync`.
- **Blocked items** get a ⛔ status marker; a **Ready** filter shows only
  unblocked open work. The preview shows the blocker chain.
- **Tabs:** Tasks (tree, default filter `status:todo,doing`), Notes, Time
  (per-day log + bar chart by tag), Tags (tracked time + open count per tag).
- **Preview:** body of selected item.

Keys (htop-ish): `j/k/↑/↓` move, `g/G` top/bottom, `⏎` toggle preview
fullscreen, `s` start/stop, `d` done, `x` drop, `a` add, `e` edit, `m`
move/re-parent, `/` filter (same selector syntax as CLI), `t` tree/flat,
`1-4` or `Tab` switch tabs, `S` sync, `?` help, `q`/`Esc` quit.

`a`/`e` suspend the TUI (leave alt screen, disable raw mode), run
`$EDITOR`, re-enter. Store reload: poll `items/` mtimes once per tick
(1 s) so changes from another shell or Obsidian show up live; `notify`
crate later if needed.

Stats definitions (shared with `cake stats --json`):

- Tracked time per day: intervals clipped at local midnight; running
  interval counts until now.
- Week: ISO week, `week_start` configurable.
- Streak: consecutive days (ending today or yesterday) with ≥ 1 min tracked
  or ≥ 1 item closed.
- Overdue: `due < today` and status in {todo, doing}.

Module layout: `src/tui/{mod.rs, app.rs (state + key handling), ui.rs
(render), stats.rs}`; `stats.rs` is pure and unit-tested, also used by
`cake stats` and `--json`.

---

## 6. Claude skill (before MCP)

`~/.claude/skills/cake/SKILL.md` (or shipped in repo under `skill/` and
symlinked). The skill teaches Claude to use the CLI with `--json`; MCP later
exposes the same operations as tools, so the skill's conventions carry over.

Contents outline:

- **When to trigger:** user mentions tasks/todos/tracking/notes/"remember
  this for the project", or a `.cake/` dir exists in the repo, or starting
  non-trivial work in such a repo.
- **Read first:** `cake ready --json` at session start and pick work from
  that list, not from everything open; `cake ls --json --status none -s
  <topic>` for context before planning; reuse existing items instead of
  creating duplicates.
- **Dependencies:** work found mid-task → `cake add … -p <current>` (subtask)
  or a `[[id]]` link in the body; ordering constraints → `cake block`. Before marking done,
  check `blocks` and report what becomes ready.
- **Write conventions:**
  - plans, designs, decisions → notes (no `--todo`), tagged with area, parented
    under the relevant task;
  - work items → `--todo`, one per deliverable, children for subtasks;
  - always `-m TEXT` or `-m -` with stdin, never interactive;
  - title = first `#` line, short; body in markdown;
  - don't create loose `PLAN.md`/`TODO.md` files in the repo when a cake
    store exists.
- **Time tracking:** never start/stop timers on behalf of the user unless
  asked.
- **Never** edit `.cake/items/*.md` directly or run git inside `.cake/`;
  always through `cake` (keeps commits, locking, sync consistent).
- **Sync:** don't run `cake sync` (network, may conflict) unless asked; if a
  command returns `{"error":{"code":"conflict"}}`, report and stop.
- **Output:** cite items by id; after changes list what was created/changed.
- Examples for the 5 most common flows (find context, add plan note, split
  task, mark done, summarise week from `cake stats --json`).

Validate with a few real sessions before freezing; then reuse the same
operation set for MCP tool definitions.

---

## 7. Implementation order (stage 1)

1. **Groundwork:** clap 4, `SelectorArgs` flatten. Old stores stay until
   step 3 replaces them, so every commit builds and works.
2. **Item format:** frontmatter parse/write (round-trip tests incl. unknown
   keys, CRLF, empty body), `Item` with status/parent/intervals and
   `blocked_by`, ISO time helpers, id generator
   + prefix resolution.
   Lenient reading of foreign files (§3 Phone): defaults, id from
   filename, no rewrite on read.
3. **Store:** directory load/save, dirty tracking, derived children and
   blocks, cycle check on re-parent and block, atomic writes, discovery
   (§2). Delete single-file md/json stores and `-i/-o`.
4. **Commands** on the new store: add/edit/start/stop/done/drop/todo/
   tag/mv/block/unblock/ready/rm/ls/show/log/stats.
5. **`--json`** for all commands + error codes.
6. **Git:** rewrite `git.rs` around `-C <dir>`, status-checked calls,
   `init` (create + attach), commit, `sync` (`--publish`, `--continue`,
   `--abort`), `MERGE_HEAD` detection, lock, `-g` global store. Delete
   stash/checkout logic. Integration tests in temp repos (bare remote +
   two clones).
7. **TUI:** stats module first (tested), then ratatui app, then live
   reload and editor suspend.
8. **Skill:** write, try in real sessions, iterate.

Stage 2: `cake mcp` — stdio MCP server, tools mirroring the skill's
operation set, sharing the JSON schema from step 5.

Stage 3: `.cake/index.sqlite` (gitignored), rebuilt incrementally from
mtimes, FTS5 over title/body/tags; `cake ls -s` and MCP `search` switch to
it. Only when file scanning gets slow.

Later / maybe: semantic merge driver, cross-store dashboard, GitHub issues
sync, recurring tasks, estimates vs actual, personal branches
`cake/<user>`, opt-in auto sync, hidden refs.

## Open decisions

- Should `start` on a child also count time for its parents in stats (rolled
  up) — display-only rollup proposed.
- Keep notes (no status) in the Tasks tab tree when they have task
  children/parents, or strictly separate tabs?
