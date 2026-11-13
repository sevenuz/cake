---
id: form-keys
status: review
tags: [tui]
parent: "[[cake-morning-terminal-form]]"
created: 2026-11-14T00:05:44+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
usage: {claude-opus-5-5: {in: 44, out: 32302, cache_read: 16806880, cache_write: 54912}}
assignee: ai
owner: sevenuz
time:
  - 2026-11-14T00:05:44+01:00/2026-11-14T00:12:13+01:00
---
# Form key grammar: vim-like, one meaning per key, configurable

Agreed with the user on 2026-11-14. Every form module follows this; a
change to the form is checked against it first.

## Model (like lazygit panels, aerc mail lists)
- modules are tabs in the header; h/l walk them, empty ones are passed
- items inside a module: j/k (rows, options), J/K (cards, like aerc's
  next/previous message), <c-d>/<c-u> scroll long text
- leaving a table with h/l saves it, like leaving a field; u undoes a row

## Principles
- vim first: motions, modes, operators and : commands mean what they mean
  in vim; domain actions take letters vim leaves free in a list
- one key, one meaning, in every module (today `o` is open in sort but
  ok in news: that is the kind of break this rule forbids)
- every decision visible and reversible until saved (`u`)
- the common path in at most 2 keys
- all bindings configurable in config `keys`, vim notation

## Normal mode
| keys | action | where |
|---|---|---|
| h l, <left> <right> | previous, next module (tables save first) | all |
| j k, <down> <up> | down, up (rows; options in cards) | all |
| J K | next, previous item | cards |
| gg G | first, last | all |
| <c-d> <c-u> | scroll the card text half a page | all |
| o <space> | open the row's text under it | tables |
| <esc> | close, cancel (never leaves the module) | all |
| <cr> | cards: confirm the chosen option; tables: save all | all |
| 1-9 | cards: choose option n (enter confirms) | cards |
| i | insert text: own answer, comment, change request | all |
| t | make it a task (stays on the row) | sort |
| n | keep it as a note (next row) | sort |
| dd x | drop (next row) | all |
| c | complete: done (next row) | review, mine |
| s | start or stop the timer | mine |
| w | who: cycle you, ai, agents, people | sort, mine, news |
| p | prio: cycle 1, 2, 3, none | sort, mine |
| u | undo the row's decision | tables |
| } | skip the rest of this goal | cards |
| f | hint keys on the tree, a key opens that item | cards |
| F | feedback on this screen (an inbox item, tag form-feedback) | all |
| : | command line | all |
| ? | all keys | all |
| q | quit (asks once when decisions are unsaved) | all |

## Command line (:)
`:due fri`, `:deadline 30.11.`, `:who ai`, `:prio 1`, `:move paper`,
`:w` save, `:q` quit, `:q!` quit without saving, `:wq` save and quit.
Dates in every form of `parse_date`; who: me, ai, names from the store
config.

## Insert mode (text input)
Typing inserts. <cr> submits. <esc> or `jk` typed quickly (within 300 ms,
like vim's timeoutlen) leaves without saving; a lone `j` followed by
another letter is inserted as text. <c-c> cancels too. Configurable.

## Configuration (config `keys`)
```
keys: {
  drop: "dd x"
  escape: "<esc> jk"
  who: "w"
}
```
Action names as in the tables above (down, up, top, bottom, half_down,
half_up, open, back, confirm, insert, task, note, drop, complete, timer,
who, prio, undo, next, previous, skip_goal, hints, feedback, command,
help, quit, escape). Several keys per action, space separated.

## How the UI is checked
- screens rendered at 60 and 100 columns, unicode and ascii, on a page
  for the user before a change ships
- scenario script (tests/form_scenarios.py): day 1 / day 2 with real
  keys in a pseudo terminal, counts keys per decision, run on every
  form change
- feedback from the form with F, collected and handled in batches

- done when: keymap implemented and configurable, all modules use it,
  scenarios pass, screen page reviewed by the user

changed: vim-like keymap in all modules (1-9 options, dd/x drop, :due/:who/:move, jk leaves text), config keys, F feedback, scenario test; commits ea3ace7, bfda786; screens: https://claude.ai/artifact/32aUj3LK2Bgds1a5zSS85h; try: cake, then ? for all keys

