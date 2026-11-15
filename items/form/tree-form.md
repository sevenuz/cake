---
id: tree-form
status: done
tags: [handoff]
parent: "[[form]]"
created: 2026-11-14T11:59:44+01:00
by: claude-code
sessions: [9dc30da3-12f6-4942-83f6-605fbb210d3a, eb1412a2-56ac-4478-9717-6ebac99000ac]
closed: 2026-11-15T23:17:24+01:00
assignee: ai
owner: sevenuz
---
# Tree view for all form modules

- why: tables split items from their parents, rows vanish after saving (h/l, then back), and the answer module cannot walk the tree, only the questions; the user loses context and input
- what: one tree widget for news, sort, answer, review, mine; each module shows a filtered subset with parents as context; values are set in an edit line under the hovered entry; changed items stay visible until quit
- design: [[tree-form-design]]; subtasks in build order (blocked-by between siblings)
- done when: all five modules use the tree, the scenario test (tests/form_scenarios.py) walks it, tables are gone from the form
