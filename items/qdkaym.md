---
id: qdkaym
parent: "[[3s7aa0]]"
created: 2026-10-27T13:31:39+01:00
by: claude-code
sessions: [69b1d969-f2c3-4f41-b58d-a996441f99e2]
---
# Review: cake status

- changed: new `cake status` (fa15b02); ~/.claude/statusline.sh calls it in the session dir
- check: in ~/git/cake run `cake status` → `review 1`; then `cake start 3s7aa0 && cake status` → `▶ 3s7aa0 cake status one-liner fo… 0:00 (claude-code) · review 1`, `cake stop 3s7aa0`
- risk: status line shells out to cake per refresh (~5 ms)
- open: prompt snippet for your bashrc is in chat, not applied
- question: why does `review 1` not count review items you created yourself?
