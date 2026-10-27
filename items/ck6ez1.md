---
id: ck6ez1
status: done
tags: [cli, review]
created: 2026-10-27T12:19:28+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2]
closed: 2026-10-27T14:49:28+01:00
priority: 2
---
# Id prefixes

CLI accepts any unique prefix (min 3) like git, display uses the shortest unique prefix. Ambiguous prefix -> error code ambiguous_id.