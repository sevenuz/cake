---
id: form-look
tags: [tui]
parent: "[[cake-morning-terminal-form]]"
created: 2026-11-13T20:22:42+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Friendlier form: centered cards, step header, cake animation

- why (user): the form works but feels raw; splitting the screen to open
  a parent is weird in half-width panes, the last result line hangs
  around, a-c commits without Enter, the keys sit at the bottom while
  the question is at the top
- decided: a parent's text opens under its line in the tree (esc
  closes); choose with selection + enter, nothing preselected, own
  words as the last option; answered items stay in the tree with ✓
  and the answer dimmed; no collapsing
- column of about 72 characters, centered, flows top to bottom; key
  line right under the card
- step header box: new tasks · QUESTIONS · check · plans with a
  progress bar over all items
- item as a card (title in the border), options with › selection
- start screen: cake with flickering candles and what waits; end
  screen: one slice eaten per finished step + summary; empty state
  "nothing waiting"; spinner while a change is saved
- ? help overlay; config `animations: false`; ascii style uses +-|
- done when: a full pass in a half-width pane reads top to bottom
  without jumping, in unicode and ascii style
