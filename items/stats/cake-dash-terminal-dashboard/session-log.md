---
id: session-log
status: task
parent: "[[cake-dash-terminal-dashboard]]"
created: 2026-11-15T22:31:55+01:00
by: claude-code
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: claude-code
owner: sevenuz
---
# Session log: your time per store from Claude and cake

- why: the user never runs timers; stats need their time per project and the Gantt needs who worked when ([[dash-design]])
- one file per store and machine: .cake/log/YYYY-MM-<host>.jsonl, lines {start,end,who,kind,agent,session,item}
- kind chat: the Claude Code hook writes session start/stop; item = the session's task in doing at that time
- kind cake: dashboard or form open
- done when: hook and form write lines; a reader merges all files of a month; tests