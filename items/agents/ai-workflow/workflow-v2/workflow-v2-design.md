---
id: workflow-v2-design
tags: [doc]
parent: "[[workflow-v2]]"
created: 2026-11-13T23:13:42+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
---
# Workflow v2 design: decisions, reasons, details

Agreed with the user in chat on 2026-11-13. Implement from this note;
the plan steps under [[workflow-v2]] only order the work.

## Goal
- the user only captures with `cake add` (any store; global or project
  is irrelevant here); everything else happens in `cake` (the form)
- the human stays in the loop for every result; agents work as long as
  they are not really blocked, but never more than the user can review
- cake itself never calls an AI; agents are started by a configured
  shell alias

## States
One status field; tags are labels only (doc, rule, goal, cli, ...).

| status | meaning | next |
|---|---|---|
| inbox | new, not sorted; from the user or proposed | sort: note, task, move, drop |
| note | knowledge, idea, goal, rule | none |
| task | to do, has an assignee | assignee starts it |
| doing | in progress, timer runs | assignee finishes it |
| review | finished, the owner checks it | ok: done, change: back to task, drop |
| done, dropped | closed | reopen with `cake do` |

Fields:
- `assignee`: who does it; real names stored (user, agent names).
  Input shorthands: `me` = the user (config `user`, else git user.name,
  else $USER), `ai` = any agent (the first agent that runs takes it).
- `owner`: who reviews; set when an item becomes a task while sorting
  (the person sorting). Subtasks inherit the parent's owner. Never an
  agent unless the user assigns it.
- `due` (soft), `deadline` (hard), `prio` 1..3; dates in every form of
  util::parse_date (fri, +3d, 30.11., 2026-11-30).

Rejected:
- `question` as a status: it is a kind, not a lifecycle step, and
  answered/working questions had no clear status. Questions are tasks.
- separate comment items (reply): they bloat the tree. Comments and
  change requests are appended to the item's text.
- review by the creator: AI subtasks would review themselves.

## Questions and comments
- an agent question = task, assignee the user, title is the question
  (ends with ?), body: 1-3 lines of context, options `- a) ...` with the
  recommended one first; it blocks the task it is about (blocked_by)
- answering = done with the answer appended ("answer: b) ..."); the
  blocked task becomes ready again
- a comment on proposed or finished work is appended as
  "comment (<user>, <date>): ..." and the item goes back to its
  assignee as `task`

## Splitting (triage) and parents
- an agent splits a task assigned to an agent that has no children:
  either work it directly, split it, or ask
- subtasks of a task assigned to an agent are created as `task` with
  assignee = that agent and owner = parent's owner. Reason: in the
  inbox they would wait for the user's next session, one day per split
  level. The user sees them in `news` and in review.
- agent notes (docs, findings) are created as `note`, never inbox
- a parent with open children shows "waits for N subtasks", is never
  ready itself, and goes to review by itself when all children are done
  (a short final review: the children's results in one place)

## When an agent may work
- ready for agent A = status task, assignee A or `ai`, nothing open in
  blocked_by (review counts as open: a change would mean rework), no
  open children
- the agent run: take the ready item with the highest score, work it,
  set review (with what changed, commits, how to try it), recompute,
  continue; on a needed decision: create the question, continue with
  the next ready item; stop when nothing is ready
- siblings without blocked_by continue while a sibling is in review
- brake `max_review` (store config, default 8, -1 off): while that many
  items wait for review by the user, `cake ready` returns nothing for
  agents and `cake start` by an agent refuses with "waiting for review"
- emergency stop against runaway: `--max-turns` in the triage alias
- an agent asks only when a decision is hard to undo or changes the
  scope; otherwise it decides and says so in its review note
- rules: notes tagged `rule`/`goal` apply to their subtree (top level:
  whole store); agents read the ones up the tree before splitting,
  working or asking ([[rule-read]])

Rejected: a cap of items per run. It does not match the real limit,
which is the user's review capacity.

## Store config (.cake/config.hjson, committed, shared)
```
people: ["sevenuz"]          // humans, for `w` and validation
agents: ["claude-code", "local"]
max_review: 8                // -1: no brake
```
Unknown assignees are refused. User config gets `user:` (override of
git user.name, e.g. a work git name) and `form:` (module order).

## Form modules
`cake` runs config `form` (default news, sort, answer, review, mine,
triage); `cake form <modules...>` runs only those, in that order.
`dump` (one item per line) stays as an optional module.

- news: what agents did since the user's last form run (state file in
  ~/.local/state/cake per store: timestamp of the last run): new
  subtasks, started, finished, new questions; per line ok (just seen),
  hold (assignee back to the user, the agent stops), drop
- sort: a table of inbox items plus tasks without assignee (layout
  below)
- answer: tasks assigned to the user that block someone else's task,
  grouped by goal with the tree overview; options a-e, the last is own
  words; enter confirms; nothing preselected
- review: items in review owned by the user: a) ok → done,
  b) drop, last: change (text appended, back to task)
- mine: the user's open tasks by score: s start, d done, x drop,
  l later
- triage: runs the `triage` alias if configured, else skipped

Sort table:
```
   kind   title                      who  due     !
 › task   renew passport             me   fri     -
   note   blog post about cake
   ·      call landlord
 j/k move  n note  t task  x drop  m move to store
 w who  d due  D deadline  p prio  o open  enter save all
```
- one key decides and moves to the next row; `t` sets who = user;
  `w` cycles user, ai, agents, people; `p` cycles 1, 2, 3, none;
  `d`/`D` open a date field
- nothing is saved before enter; undecided rows stay in the inbox
- on a parent row, a decision can apply to the whole subtree (open
  question to the user: which key)

## Triage alias (user config template)
```
triage: "!claude -p --max-turns 40 --allowedTools 'Bash(cake *)' 'Run the agent run of the cake skill for this store: apply answers and comments, split and work tasks assigned to ai until nothing is ready or reviews are full, then a short summary.'"
```
Aliases starting with `!` run a shell command, like git. Code work
needs more tools than `cake *`; the template comment lists them.

## Migration (`cake migrate`: overview first, then apply)
| old | new |
|---|---|
| status none (note) | note |
| tag ask, open | task, assignee user, keeps blocking its parent |
| tag reply, open | text appended to the parent, item removed |
| tag review | status review, owner user |
| agent note with `done when`, not approved | inbox |
| todo/doing by an agent | task/doing, assignee that agent |
| todo/doing by the user | task/doing, assignee user |
| tags triaged, sorted | removed |

## Example
Day 1: the user captured "renew passport", "blog post about cake",
"sync errors unclear" with cake add. `cake`: sort: passport task (move
to the global store), blog note, sync errors task; assign: passport me
due fri, sync errors ai deadline 30.11. prio 2; triage runs.
Agent run: sync errors is split into "name the failed git step" (task,
ai) and "hint per error type" (task, ai); question "show git's raw
message too?" (task for the user, blocks the hint task); works on the
first → review.
Day 2: news shows the split; answer: b; review: first subtask ok;
triage: the hint task runs. Day 3: review: change "shorter hints",
back to task; once ok, "sync errors" goes to review by itself.
