---
id: gantt-doc
tags: [doc]
parent: "[[gantt-rework]]"
created: 2026-11-20T21:56:15+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
owner: sevenuz
---
# Gantt critical path design

Context: the gantt view 3 (critical path) shows one goal note with a deadline at a time and starts every task at "now", so on the phd store all bars are one block at the left and only one deadline line shows.
Why: it cannot answer "what do I do next, and does it fit every date I promised".

Decided (user, 2026-11-20):
- Targets: every open item with a deadline or a due, in the whole store, not only notes tagged goal. Goals sort first among targets with the same date.
- One view for the whole store, one block per target, ordered by date. Inside a block the rows are the target's open tasks (its subtree; a leaf target is its own row).
- Rows are selectable with j/k across all blocks and do what selected items do elsewhere in the dashboard: the card of the item under the gantt, enter opens the tree at it (`cake form all --at <id>`), e edits it.
- No capacity model: the user does one task at a time, in clock time. Hours per day are parked in [[gantt-capacity]].
- Agents (assignee ai or an agent name) run in parallel, at most max_review (store config, default 8, -1 no limit) at once: the review brake. Each agent task is followed by a review step in the owner's queue (the user's lane), so the path shows the user as the bottleneck (decided 2026-11-20).

Scheduling, one pass over the whole store so every block shows the same plan:
- The user's lane: their own tasks, unassigned tasks and the review steps of agent tasks, one after another. Other people each get such a lane.
- Agent tasks: start after their blockers and when fewer than max_review agent tasks are running or waiting for review; a slot frees when the review step ends.
- A review step: the median time from review to done of agent tasks in the history, else 15 min; already in review: only the step.
- Order in the user's lane and of waiting agent tasks: earliest target date first (deadline, else due, of the nearest target above the task or the task itself), then score; a task never starts before its blockers end.
- Length: own estimate, else the history median (shown with ~), else 1h.
- A plan (task with work children) has no length of its own: a summary bar from its first child's start to its last child's end.
- A question (task assigned to the user that blocks another) takes the median time the user needed to close questions (created to closed), else 1 day; drawn as ◆ plus a dim waiting bar.

Drawing:
- Shared time axis for all blocks, labeled in days (hours when everything fits in a day); from now to the latest planned end or target date within 4 weeks. Targets further out are written at the right edge as "→ 2027-06-03".
- Every target is a vertical line in its own block; a target nested in another target's subtree is a marker inside the outer block (no block of its own, its rows highlighted).
- Deadline line solid, due line dashed. Color by slack (target date minus planned end of its subtree): green more than 3 days, yellow 0 to 3 days, red late. The block header shows the target, date, slack ("2 d left", "5h late").
- The critical path of a block (the chain that ends last, through blockers and lane order) in the mine color.

Edge cases:
- Inbox items with a date count as targets (dim title), their subtree is usually empty: line only.
- A task with its own deadline inside a goal: marker in the goal block, not a second block.
- Cycles in blockers: cut at depth 64 as now.
- No targets: "no item with a deadline or due".

Example (phd store): blocks "osdi · deadline 12-08", inside markers for submit-register (deadline 12-01) and the goal's due 12-04; the fix-/fmt- tasks of the user run one after another from now, so the path shows how many hours of work are queued before 12-01.

Not: per-goal pages with j/k between goals (old view); working hours per day (parked); scheduling across stores.
