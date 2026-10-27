---
id: pscs0p
tags: [design, doc]
created: 2026-10-27T12:28:19+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2]
---
# Design: data model and store discovery

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

## Store discovery

Order:

1. `--store <path>` flag / `CAKE_STORE` env.
2. Walk up from `$PWD`: first dir containing `.cake/items/` → project store.
3. Inside a git repo whose remote has `cake/main` but no `.cake/`
   worktree yet → hint to run `cake init` (attach, see [[56gwt9]]).
4. Fallback: global store `$XDG_DATA_HOME/cake` (`~/.local/share/cake`),
   itself a plain git repo on `main` (optional remote).

`cake -g ...` forces the global store from anywhere, also inside a project. Config keeps a list of known stores
(appended on init) for cross-store stats later.
