---
id: handoff
tags: [doc, handoff]
created: 2026-11-14T01:17:36+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, 979940ed-e0c4-405e-b509-b734038be1bb]
---
# Handoff: state of cake and how to continue (2026-11-14)

Read this first in a new session, then `cake show workflow-v2-design` and
`cake show form-keys`. Everything here is current as of commit 4e7ee97.

## The user and how they work
- Julius (git user sevenuz), Rust, tmux with two panes: left pane runs
  `cake`, right pane runs Claude Code. Look at the left pane with
  `tmux capture-pane -p -t %6 -S -120` when they say "check the pane".
- They talk in caveman mode with Claude (a plugin hook); answers are
  terse but complete. Code, commits and items are written normally.
- Questions: short but complete, options a/b/c below, recommended first,
  a context line only when needed (memory question-format).
- Notify: when a turn ends with questions or after longer work and
  their pane is not active (`tmux display -p -t "$TMUX_PANE"
  '#{pane_active}'` is 0), send `notify-send -u critical -a cake "..."`.
- No claude.ai artifact pages for UI review; they test in the terminal.
- cake must never call an AI itself; agents drive it from outside
  (memory cake-no-ai-inside).
- They want the command surface minimal but complete, vim-like keys,
  and readable output; they push back on vague plans, so items must be
  readable without the chat (why / what / done when).

## Repo and build
- /home/jul/git/cake, branch main, push only when asked. Commit style:
  `vc: <summary>` with a body, ending with the Co-Authored-By line.
- `~/.local/bin/cake` is a symlink to target/release/cake. After code
  changes run `cargo build --release`, but never while their `cake` form
  runs (check `pgrep -x cake`): the running form calls itself through
  current_exe (there is a fallback for a replaced binary, but the old
  process keeps the old logic).
- Tests: `cargo test` (lib ~96, tests/commands.rs, json.rs, sync.rs,
  form.rs). tests/form.rs runs tests/form_scenarios.py: a day played
  with real keys in a pseudo terminal, checks the store afterwards;
  needs python3, ~25 s. Tests isolate XDG_CONFIG/DATA/STATE_HOME.
- Screen dumps for inspection: `CAKE_SCREENS=dir CAKE_STYLE=unicode cargo
  test --lib dump_screens -- --ignored` writes every module as text.
- Agents in this session run cake with AI_AGENT=claude-code in the
  environment; for user-like calls use `env -u AI_AGENT cake ...`.

## Code map (src/)
- lib.rs: CLI (clap), alias expansion (`!` aliases run sh), dispatch,
  json reports (BRIEF fields for ls/ready), commit after writes.
- commands.rs: add, edit, list/tree, show, status, set_status, comment,
  to_note, mv/mv_to_store, init, sync wrappers, reviews_full (brake).
- store.rs: Store (items in a dir tree: children of foo.md in foo/),
  load/save with moves and pruning, relations, ready rule
  (work_children, settle: parent to review), score with ScoreWeights.
- item.rs: Item, frontmatter (YAML), Status (inbox/task/doing/review/
  done/dropped, None = note; "todo" still parses), Usage.
- config.rs: Config with commented hjson TEMPLATE (written when the
  config is missing, refreshed as config.default.hjson), StoreConfig
  (.cake/config.hjson: aliases, people, agents, max_review), who()
  resolution (me, ai, names), default aliases.
- form/: mod.rs (runner, Ui, paint, Screen, header, still screens,
  dump, feedback), card.rs (answer, review), table.rs (news, sort,
  mine), keys.rs (keymap, vim notation, jk escape).
- stores.rs (stores.json, atomic writes), style.rs (unicode/ascii
  glyph table), theme.rs (theme.hjson colors), markdown.rs (own
  renderer), view.rs (show, ratatui pager), usage.rs (tokens, Claude
  Stop hook, pending file in ~/.local/state/cake), footprint.rs
  (energy/CO2e/water estimate, sources in the config template),
  migrate.rs (old tag workflow → v2; hidden command), util.rs (ids,
  slug_id, parse_date, parse_duration, find_store).

## Data model (workflow v2, see workflow-v2-design)
- status: inbox → note | task → doing → review → done | dropped.
- fields: assignee (who does it; me = the user, ai = any agent), owner
  (who reviews; set when it becomes a task, subtasks inherit), due,
  deadline, priority, blocked_by, tags (labels only), usage, sessions.
- ready = task/doing, no open blockers, no open work children (children
  that also block the parent, e.g. questions, do not count). A parent
  whose work children are all closed goes to review by itself.
- questions = tasks for the user that block the agent's task, options
  `- a) …` in the body; answer via set done -m "answer: …".
- comments are appended to the body (`comment (who, date): …`) and put
  review/doing back to task.
- agents: `ls --ready` gives their own and `ai` tasks; with max_review
  (store config, default 8, -1 off) reviews waiting, nothing, and start
  fails with code waiting_for_review.

## Commands (14 visible)
add, edit, show, ls, status, start, stop, set, form, mv, rm, sync,
init, config. Default aliases: done, drop, do/task/todo/reopen, note,
review (all `set ...`), ready (`ls --ready`), check (`ls --status
review`), inbox. Hidden: usage (add, claude-hook), migrate.
edit takes several ids and --tag a,~b, --blocked-by x,~y, --parent,
--comment, --assignee, --owner, --prio, --due, --deadline, -m.

## The form (`cake`, `cake form <modules>`)
- modules (config form): news, sort, answer, review, mine; dump
  optional. Triage is not part of the form (removed in f18d9f1): run
  `cake triage` (the user's alias) by hand or from cron. h/l walk
  modules (tables save when left), empty ones are passed unless walked
  to with h/l; an end screen waits for q. Nothing for the user at
  all (no dump module): no screen, a message how many ready tasks
  wait for agents (bd335c7).
- the user's triage alias: claude -p '<prompt>' --max-turns 40
  --allowedTools 'Bash(cake *)' (prompt must come before
  --allowedTools, which takes several values).
- keys: see form-keys; options 1-9, i text, t/n/dd/x/c/s/w/p, u undo,
  J/K items, <c-d>/<c-u> scroll, f hints, A subtree, F feedback (a task
  for ai, tag form-feedback), : commands (:due :deadline :who :prio
  :move :w :q :q! :wq), ? help; jk or <esc> leaves text input.
- config: form_width (default 100), keys (rebind actions).

## Files outside the repo
- ~/.config/cake/config.hjson (the user's, commented, triage alias),
  config.default.hjson (reference), theme.hjson (dim is gray),
  stores.json (known stores; only `cake` so far).
- ~/.claude/settings.json: Stop hook `cake usage claude-hook` (tokens
  per item); ~/.claude/skills: the cake skill is skill/SKILL.md here.
- project store: /home/jul/git/cake/.cake (worktree, branch cake/main,
  remote origin). Migrated to v2 on 2026-11-13.

## Open, in order of what the user cares about
1. Reviews waiting: command-surface, prio-column-header,
   command-line-inline (all from this session).
2. Form feedback from the user (tasks for ai, tag form-feedback):
   what-does-do-assign / news-who-fix (w in news shows "assign" but no
   who column; agent proposed showing →ai), i-lost-my-input /
   lost-input-which (sort rows vanish after h/l saves; question waits
   for the user), how-do-i-set (done: prio column and inline :).
3. Known rough edges: wrapped bullet lines lose their indent in
   markdown.rs; at 60 columns the key line and table titles are cut
   (proposal: show only dates that are set below 80 columns); the help
   overlay is long and could be grouped.
4. Dashboard (cake-dash-terminal-dashboard, assigned ai): sections
   running / waiting on you / due, time as a pie per goal, period
   switch; comes after the form, easy to skip.
5. Older backlog (claude-code tasks): session start hook (qzf5dz),
   rules/goals (kqejfn), sync error messages (sync-errors-that-say),
   merge driver (fq5jbv), stats/TUI items, MCP server, Termux widget.
6. Ideas not built: nightly Claude routine (dropped by the user for
   now; triage alias instead), local model later, migrate can go once
   all stores are v2.

## How to continue
- `cake ls --status review` and `cake ls --assignee ai` to see what
  waits; work tasks assigned to claude-code/ai the user approved; put
  results in review with `cake review <id> -m "changed …; commit …;
  try: …"`.
- After a form change: run the scenario test, rebuild release when the
  user's form is closed, mention what to try.

