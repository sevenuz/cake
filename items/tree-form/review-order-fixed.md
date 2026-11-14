---
id: review-order-fixed
status: task
parent: "[[tree-form]]"
created: 2026-11-14T20:51:27+01:00
by: claude-code
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10]
assignee: ai
owner: sevenuz
---
# Review rows keep their order after accepting

- why: accepting a row in review moves it; the user loses their place
- rows keep the order they had when the module opened; a decided row stays where it was
- sorting later only on explicit request (own key), never as a side effect
- done when: accept/drop in review leaves every row in place; scenario test
