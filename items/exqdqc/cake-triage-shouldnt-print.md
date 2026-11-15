---
id: cake-triage-shouldnt-print
status: task
parent: "[[exqdqc]]"
created: 2026-11-14T11:22:41+01:00
by: sevenuz
sessions: [eb1412a2-56ac-4478-9717-6ebac99000ac]
assignee: ai
owner: sevenuz
---
cake triage shouldnt print all the output from the ai because this is in the store now. maybe just report how many tokens and how long it took, maybe a one liner from the ai. should we also report our triage usage somewhere?

here example output how it is now, verbose:

jul@julspc:~/git/cake(main)$ cake triage
I split the three tasks assigned to ai and applied your one new answer. Nothing was implemented, and I didn't run `cake sync`.

**Answer applied**
- `what-does-do-assign`: added `news-drop-w` "Remove w from the news table", following your answer c in `news-who-fix`. It's ready for an agent.

**Split into subtasks**
- `here-parent-description-useful` (your feedback on the mine table and the feedback box):
  - `overlay-wrap-cursor`: the feedback box works out its height and cursor position from character count, but the text wraps at word boundaries. So the new line stays hidden and the cursor sits in the wrong place. Ready.
  - `overlay-arrow-keys`: arrow, home/end and delete keys to move and edit within the text, also for the `:` input line. Waits for the item above.
  - The parent task also waits for your answer to question `how-set-parent-headings`.
- `dashboard-we-should-have` (section for the last day, switchable to week, month or year):
  - `time-per-goal`: adds up tracked time per goal for a period. Ready.
  - `dash-period-key`: one key cycles day, week, month and year. I proposed `p`, because `f` already shows the hint keys in the form; please confirm or change it when you review. Waits for `time-per-goal`.
  - The parent task also waits for your answer to question `last-day-section-time`.

**Questions for you**
1. `how-set-parent-headings`: how should parent headings stand out in the tables? a) arrow marker with its own color (recommended), b) blank line and underline, c) a thin line around the title.
2. `last-day-section-time`: should the last-day section show a time pie chart, a list of what got done, or both? a) both (recommended), b) pie only, c) list only.

**Ready for an agent now:** `news-drop-w`, `overlay-wrap-cursor`, `time-per-goal`.

The desktop notification didn't go out: the check for whether your terminal pane is active needed permission.

