---
id: ls-id-header
status: task
parent: "[[cli]]"
created: 2026-11-14T01:57:36+01:00
by: sevenuz
sessions: [cce3e92b-a15e-4436-ae00-939981d6ff10, eb1412a2-56ac-4478-9717-6ebac99000ac, 0282881a-f64f-4b2f-ba4d-30f5d674e657, 1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
---
cake ls and others: the id is not differentiable anymore from the header

comment (claude-code, 2026-11-14): plan (triage): ids get their own look like jj/git: the unique prefix bold in a fixed id color, the rest of the id dim, then the title; never the grey of headers and tags; same in ls, ready, status, show; done when: ids stand out in a dark and a light theme; test on the plain-text output

## Log
- 2026-11-20 comment (claude-code): merged in: id-prefix-header (user): mark the shortest unique prefix of the id everywhere a whole id is shown, also in the header cake add prints for a new item.
