---
id: form-last-module
status: task
parent: "[[tui-consistency]]"
created: 2026-11-21T02:24:45+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
assignee: ai
owner: sevenuz
estimate: 1h
---
# Form: last module and dashboard F

Context: l on the last module of a form opened from the dashboard (e.g. `form mine`) leaves the form like q; dashboard F runs a hard-coded news/inbox/answer/review (no mine, no ALL tab, config `form` ignored); on the all tab F/T/D/a act only on the first store without saying so.
Change: on the last module l shows the note the ALL tab shows ("the last tab: q closes the form"); F runs the configured form; on the all tab pass all stores, or name the target store in the footer.
Done when: l never closes the form; F follows config `form`; test.
