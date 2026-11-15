---
id: dash-design
tags: [doc]
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-15T22:31:39+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
owner: sevenuz
---
# Dashboard design (grilling 2026-11-15)

## Purpose
- why: the user works on several cake projects a day, dumps ideas everywhere and wants to know what got done, how long, in which project, tokens and environmental footprint per project
- live view of running work, to parallelize: see what agents do while they run
- tell the user where to put their focus next, show what the AI does next
- bare `cake` opens the dashboard; the form is entered from it (`cake form` stays)

## Stores
- all stores from stores.json, one tab each; the nearest store first and selected at start; an `all` tab with the same sections across stores (lines prefixed with the store)
- no config switch for this (user: not needed)

## Main screen (per store tab)
- header from draft A: tab line `all · CAKE · habitats · personal`, right side today's totals `5h20 · 1.2M tok · 0.4 kg CO2e`
- sections: running (doing items with timers, agent, elapsed), your focus, ai queue (next 3-5 ready agent tasks by score, `+N`, and what stops the rest: waiting on you / review brake)
- your focus order: answers that block agents → reviews (brake) → deadlines → own tasks by score; plus a warning line when a goal's critical path misses its deadline ("stats: 2h over the deadline")
- card at the bottom for the selected line, like the form
- keys: h/l store, j/k line, <cr> opens the form at that module and store, back in the dashboard after the form; p pie, s stats, g gantt, ? keys, :q quit
- live: re-read all stores every 1-2 s (file mtimes), redraw
```
╭───────────────────────────────────────────────────────────────────────────────╮
│ all · CAKE · habitats · personal           today 5h20 · 1.2M tok · 0.4 kg CO2e │
╰───────────────────────────────────────────────────────────────────────────────╯
 running
   ◐ archive-old  Archive old closed items               claude-code     0:42
 your focus                                          (unblocks agents first)
 › ? 2 answers block agent work          → 35xst1, kqejfn
   ◈ 8 reviews, limit hit                → agents stopped
   ! habitats-report  deadline Fri
 ai queue                                            19 ready
   35xst1  Stats module behind cake stats                  59
╭ Archive old closed items ─────────────────────────────────────────────────────╮
│ running 0:42 · claude-code · 18k tok · started 14:02                          │
╰───────────────────────────────────────────────────────────────────────────────╯
 h/l store  j/k  <cr> form here  p pie  s stats  g gantt  ? keys  :q quit
```

## Overlays (full screen, tab line stays, live)
- pie (p): one pie in braille (~14x7 chars) with legend; m cycles measure: time / tokens / done items; d cycles period day → week → month → year; <cr> on a slice zooms from stores into that store's goals, <bs> back; legend line "you 2h10 · agents 3h10"; not shown at start for now (later maybe as intro once a day)
- stats (s): table per store: you, agents, tokens, kWh, CO2e, done, per-day sparkline; total row; overdue, reviews waiting, open questions, streak; d period
- gantt (g): v cycles G1 → G2 → G3; d: day in hours (default) ↔ week (one column per 2 h)
  - G1 (default) lanes per session: you + each agent session; bars in the store's color; `···` where an agent waited on the user (question or review), `▒` user in the cake form; footer: max/avg parallel, total agent wait on the user. Answers "who worked when, where was I the bottleneck"
  - G2 lanes per store, shade = sessions at once: ░ 1, ▒ 2, ▓ 3, █ 4+, `⁚` an agent waits on the user. Shows that waiting happened, not why (G1 shows the user's lane busy elsewhere)
  - G3 items: classic gantt with blocked_by; pick a goal with a deadline → forward view now ┊ … │ deadline: open tasks chained by blocked_by with durations, questions as ◆ on the path; critical path line "a → b → c = ~6h agent + your answer · fits ✓", red and "N h over" when not
```
            09    10    11    12    13    14    15    16
 you        ████████████▒▒▒▒▒            ██████████
 cc · cake  ████████████                 ████████████████
 cc · hab      ███████······█████              ██████████
```

## Data
- item time: timers on items as now (agents' start → review)
- session log per store: `.cake/log/YYYY-MM-<host>.jsonl`, one file per machine so no two writers share a file (no merge conflicts, no git tricks); stats read all files of a month
  - line: {"start","end","who","kind":"chat"|"cake","agent","session","item"}
  - kind chat: Claude Code session time from the hook (start/stop), counts as the user's time too; `item` set while the session has a task in doing, else store only ("talking and planning")
  - kind cake: time with the dashboard or form open
- durations for the critical path: new estimate field (`--est 2h`), set by agents when they split tasks; fallback: median tracked time of done tasks of the same kind (agent or user)
- tokens and footprint per item exist (usage, footprint config)

## Rejected
- dashboard only for the nearest store; `-g` meaning all stores (overloads -g)
- timers as the user's time source (the user never runs them)
- hidden `_chat` / dot items for the log: one shared file per store conflicts on sync, interval format has no who/kind/session, grows forever
- one shared log file with merge=union (works, but per-machine files need no git rule)
- Gantt as its own tab (full-screen overlay loses nothing)
- spreading session time over all touched items (a guess)

## Build order
1. session log + estimate field + stats core (time, tokens, footprint per store, goal, period; clipped)
2. dashboard: store tabs, Now sections, <cr> into the form, bare cake
3. pie and stats overlays
4. gantt G1 → G2 → G3 with critical path
5. live reload / editor suspend
