---
id: json-new-for
status: task
parent: "[[agents]]"
created: 2026-11-21T01:39:42+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
estimate: 3h
---
# JSON trimming and what is new for me, in cake

Context: agents piped `cake --json` into short Python scripts to cut fields, read the end of the log and find items with new comments.
Why: that logic belongs in cake, works for every agent and needs no Python.
Change: `--fields id,status,...` keeps only these keys per item (id always); selector `--new-for WHO`: items someone other than WHO changed after WHO's last change (from the store's git history; me = the user, ai = any agent, a name = that person or agent); `show <id> --diff [WHO|DATE]`: the item's diff since WHO's last change or since DATE (default: the caller).
Done when: tests cover the three; the skill uses them instead of scripts.
Not: a `show --log N` (the history is in git, works for any body format).
