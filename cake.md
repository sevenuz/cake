| id | cake|
|---|---|
| timestamp | Tue Oct 17 09:29:58 2023 +0200|
| last modified | Thu Apr 24 12:23:31 2025 +0200|
| tags | cake|
| timetrack | |
| parents | |
| children | 109, 594, c3e, 5dc, d0a, 67d, 5d1, 74e, 91d, f70, d27, dfa, ath, urd, rdt, lsh|

# Cake TUI
A tool to organize todos with
- tags
- ids
- and relations like parents and children
 - recursion is allowed and marked so
- timetracking
- sync with git repo
- serialization in two formats
 - markdown as default
 - json
- settings in json in ~/.cake
- default cake file ist there as well
 - finds nearest cake file from pwd
- markdown presenter for terminal in general
- rich stats, especially cake charts ;D

---
---
---
| id | 109|
|---|---|
| timestamp | Tue Oct 17 09:48:24 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

recursion support for r, rr, rrr

---
---
---
| id | c3e|
|---|---|
| timestamp | Tue Oct 17 09:50:30 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | Tue Oct 17 17:06:29 2023 +0200, Tue Oct 17 17:13:12 2023 +0200|
| parents | cake|
| children | 70f|

# config file
should contain color sceme for markdown
default saving location
aliases?
default selectors
defualt modifiers for one or multiple hits
hide_recursive_elements

---
---
---
| id | 594|
|---|---|
| timestamp | Tue Oct 17 09:53:49 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

code cleanup!

---
---
---
| id | 70f|
|---|---|
| timestamp | Tue Oct 17 17:34:08 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | c3e|
| children | |

run cmd

---
---
---
| id | 5dc|
|---|---|
| timestamp | Tue Oct 17 19:03:20 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

make view scrollable

---
---
---
| id | d0a|
|---|---|
| timestamp | Tue Oct 17 19:04:40 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

make order deterministic

---
---
---
| id | 67d|
|---|---|
| timestamp | Wed Oct 18 09:14:06 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

format for long: timetrack

---
---
---
| id | 5d1|
|---|---|
| timestamp | Wed Oct 18 15:35:48 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

tests

---
---
---
| id | 74e|
|---|---|
| timestamp | Wed Oct 18 15:45:22 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

save timediff as second entry for timetracking

---
---
---
| id | 91d|
|---|---|
| timestamp | Sat Oct 21 23:40:10 2023 +0200|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | wichtig, cake|
| timetrack | |
| parents | cake|
| children | |

summe im timetrack

---
---
---
| id | f70|
|---|---|
| timestamp | Wed Feb 28 00:51:06 2024 +0100|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

exclustion with ~ for ids, parents and children as well

---
---
---
| id | d27|
|---|---|
| timestamp | Wed Feb 28 11:28:04 2024 +0100|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

deadline date?

---
---
---
| id | dfa|
|---|---|
| timestamp | Thu Feb 29 18:19:41 2024 +0100|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

id collision on big todo generation

---
---
---
| id | ath|
|---|---|
| timestamp | Thu Feb 29 18:19:59 2024 +0100|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

stress test for big todo files

---
---
---
| id | urd|
|---|---|
| timestamp | Wed Mar  6 11:33:40 2024 +0100|
| last modified | Thu Apr 24 11:52:37 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

limit chars for long lines in short ls

---
---
---
| id | rdt|
|---|---|
| timestamp | Thu Apr 24 11:54:52 2025 +0200|
| last modified | Thu Apr 24 12:14:18 2025 +0200|
| tags | cake|
| timetrack | |
| parents | cake|
| children | |

# git support
`cake init --git (--remote)` creates branch `cake` (can be configured of course) and there it creates
`cake.md`, adds it and pushes it (--set remote host...).

Then, if you use all normal commands, it should check `git branch` for a `cake` branch,
if so:
- stash
- checkout cake
- fetch cake -> only if --remote branch exists
- rebase cake -> only if --remote branch exists
- all cake operations here
- commit with message of changed or new ids
- push -> only if --remote branch exists
- checkout previous branch
- stash pop

on conflicts which should not happen because the instant sync:
- abort operation and print message: solve conflicts on `cake` branch

---
---
---
| id | lsh|
|---|---|
| timestamp | Thu Apr 24 12:18:14 2025 +0200|
| last modified | Thu Apr 24 12:23:31 2025 +0200|
| tags | |
| timetrack | |
| parents | cake|
| children | |

# Charts and statistics
if no subcommand is given, show rich charts
- use https://ratatui.rs/
- use https://github.com/Lol3rrr/termgraph ???

Statistics like:
- last added items
- items with most children
- deepest chain
- recursion count
- time overview
- tag stats

Graph:
- show graph of items
