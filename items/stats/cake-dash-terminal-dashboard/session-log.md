---
id: session-log
status: review
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
usage: {claude-opus-5-5: {in: 48, out: 19055, cache_read: 4206304, cache_write: 33770}}
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-15T22:32:04+01:00/2026-11-15T22:37:30+01:00
---
# Session log: your time per store from Claude and cake

- why: the user never runs timers; stats need their time per project and the Gantt needs who worked when ([[dash-design]])
- one file per store and machine: .cake/log/YYYY-MM-<host>.jsonl, lines {start,end,who,kind,agent,session,item}
- kind chat: the Claude Code hook writes session start/stop; item = the session's task in doing at that time
- kind cake: dashboard or form open
- done when: hook and form write lines; a reader merges all files of a month; tests

changed: new log module; Stop hook (already installed) writes kind chat lines: transcript user/assistant times → stretches (pause ≤15 min joins, continues the last stretch), split by the timers of the session's items (item set while a task ran); form writes kind cake lines from key presses on exit (hidden cake usage log); files .cake/log/YYYY-MM-<host>.jsonl; no own commit per hook call (would be one per answer): logs ride with the next item commit, sync commits them first; commits: e02d67f; try: chat a bit, then cat .cake/log/*.jsonl
