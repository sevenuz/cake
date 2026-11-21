---
id: rule-read
status: review
parent: "[[rule-notes]]"
created: 2026-10-27T14:51:01+01:00
by: claude-code
commits: [9fa6d3b, caaaa80]
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2, 81954e9f-9185-4d55-a366-5581de2aacd6, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: claude-code
owner: sevenuz
time:
  - 2026-11-20T23:07:41+01:00/2026-11-20T23:09:54+01:00
  - 2026-11-21T01:39:42+01:00/2026-11-21T01:46:21+01:00
tokens_in: 24
tokens_out: 6456
tokens_cache_read: 2223383
tokens_cache_write: 12411
---
# Rule and goal notes that agents read first

- why: some things hold for a whole project or one area ("no new
  dependencies", "paper is due 2026-12-01"); today you repeat them in
  every chat, agents forget them between sessions
- you write them as normal notes tagged `rule` or `goal`, no new fields
- where they apply: a top-level note → the whole store; a note under X →
  X and everything below it
- before working on an item the agent reads the rule/goal notes on the
  way up to the top (the skill says so)
- later maybe: `cake rules <id>` prints that list
- done when: the skill does it, and 1–2 real rules exist in this store

## Log
- 2026-11-20 changed: new command cake rules [ID]: top-level rule notes, then on the way down to ID each goal note and the rule notes filed directly under each step; without ID the top-level rules and goals. The skill uses it in the agent run (step 2) and at session start; it notes that -t rule,goal means both tags (that is why agents found nothing). First real rule: rule-principles (from AGENTS.md). try: cake rules gantt-view; cake rules
- 2026-11-21 discuss (sevenuz): do we really need a new command for it? what is the advantage over a alias?
- 2026-11-21 comment (claude-code): chat 2026-11-21: option b: general pieces (an ancestor-path selector, an either-tag form for -t) and rules as a built-in alias; names still open
- 2026-11-21 changed: rules is no command anymore but a built-in alias, 'ls -l --status note -t rule/goal --along'; new general pieces: --along ID (top level, the path down to ID and the direct children of each, lists only the matches) and -t a/b (either tag). Difference to before: top-level goals off the path now come along too. try: cake rules gantt-view; cake ls -t rule/goal --along form-polish
