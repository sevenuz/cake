| id | 37e|
|---|---|
| timestamp | Thu Mar  2 16:29:18 2023 +0100|
| last modified | Mon Oct 16 21:52:00 2023 +0200|
| tags | alt, done, nice|
| timetrack | Mon Oct 16 21:51:53 2023 +0200, Mon Oct 16 21:52:00 2023 +0200|
| parents | |
| children | |

first of the new format

---
---
---
| id | easycase|
|---|---|
| timestamp | Tue Mar  7 13:53:04 2023 +0100|
| last modified | Sat Oct 21 23:39:21 2023 +0200|
| tags | done, nice|
| timetrack | Tue Mar  7 13:55:06 2023 +0100, Tue Mar  7 13:55:20 2023 +0100, Tue Mar  7 13:56:42 2023 +0100, Tue Mar  7 13:56:47 2023 +0100, Mon Oct 16 21:51:53 2023 +0200, Mon Oct 16 21:52:00 2023 +0200, Tue Oct 17 09:23:05 2023 +0200, Tue Oct 17 09:29:44 2023 +0200, Tue Oct 17 17:03:37 2023 +0200, Tue Oct 17 17:13:12 2023 +0200, Fri Oct 20 18:31:43 2023 +0200, Fri Oct 20 18:31:55 2023 +0200, Sat Oct 21 23:30:49 2023 +0200, Sat Oct 21 23:30:57 2023 +0200, Sat Oct 21 23:39:03 2023 +0200, Sat Oct 21 23:39:21 2023 +0200|
| parents | frech|
| children | a76, 2c5|

# EasyCase 
morgen wird fleißig geeasycased von zu hause, das wird mega :)

## subheader 1
```
fn burger() {
//some stuff
}
```
### subheader 2
[link](https://hayrave.de)

---
---
---
| id | a76|
|---|---|
| timestamp | Tue Mar  7 13:53:47 2023 +0100|
| last modified | Mon Oct 16 22:59:21 2023 +0200|
| tags | |
| timetrack | Tue Mar  7 13:55:06 2023 +0100, Tue Mar  7 13:55:20 2023 +0100|
| parents | easycase|
| children | |

item api mit hooks

---
---
---
| id | 2c5|
|---|---|
| timestamp | Tue Mar  7 15:13:35 2023 +0100|
| last modified | Mon Oct 16 22:59:21 2023 +0200|
| tags | tectrixer|
| timetrack | |
| parents | easycase|
| children | frech|

sveltekit

---
---
---
| id | frech|
|---|---|
| timestamp | Mon Oct 16 21:44:51 2023 +0200|
| last modified | Sat Mar  9 12:44:07 2024 +0100|
| tags | |
| timetrack | |
| parents | 2c5|
| children | easycase, ntj|

recursivooo

---
---
---
| id | cake|
|---|---|
| timestamp | Tue Oct 17 09:29:58 2023 +0200|
| last modified | Wed Mar  6 11:33:40 2024 +0100|
| tags | |
| timetrack | |
| parents | |
| children | 9d7, 109, de8, ef5, 594, a60, 713, c3e, 5dc, d0a, 67d, 5d1, 74e, 91d, f37, f70, 5e4, d27, dfa, ath, urd|

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
| id | 9d7|
|---|---|
| timestamp | Tue Oct 17 09:47:35 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

markdown serialization

---
---
---
| id | 109|
|---|---|
| timestamp | Tue Oct 17 09:48:24 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

recursion support for r, rr, rrr

---
---
---
| id | de8|
|---|---|
| timestamp | Tue Oct 17 09:49:08 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

beautify short view

---
---
---
| id | ef5|
|---|---|
| timestamp | Tue Oct 17 09:50:11 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

show cmd to show md files in gerenal

---
---
---
| id | c3e|
|---|---|
| timestamp | Tue Oct 17 09:50:30 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | Tue Oct 17 17:06:29 2023 +0200, Tue Oct 17 17:13:12 2023 +0200|
| parents | cake|
| children | 70f, d4f|

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
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

code cleanup!

---
---
---
| id | b42|
|---|---|
| timestamp | Tue Oct 17 14:41:36 2023 +0200|
| last modified | Tue Oct 17 14:41:36 2023 +0200|
| tags | project|
| timetrack | |
| parents | |
| children | |

gome pdf view with tabs

---
---
---
| id | a60|
|---|---|
| timestamp | Tue Oct 17 17:30:59 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

remove other color lib

---
---
---
| id | 70f|
|---|---|
| timestamp | Tue Oct 17 17:34:08 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | |
| parents | d4f, c3e|
| children | |

run cmd

---
---
---
| id | 713|
|---|---|
| timestamp | Tue Oct 17 17:35:00 2023 +0200|
| last modified | Thu Feb 29 18:01:19 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

uuid alphabet to keyboard homerows

---
---
---
| id | d4f|
|---|---|
| timestamp | Tue Oct 17 17:36:02 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | c3e|
| children | 70f|

in the middle

---
---
---
| id | 5dc|
|---|---|
| timestamp | Tue Oct 17 19:03:20 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
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
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

make order deterministic

---
---
---
| id | 7d2|
|---|---|
| timestamp | Tue Oct 17 19:07:04 2023 +0200|
| last modified | Tue Oct 17 19:07:04 2023 +0200|
| tags | project|
| timetrack | |
| parents | |
| children | |

gnome-translate-indicator

---
---
---
| id | 67d|
|---|---|
| timestamp | Wed Oct 18 09:14:06 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
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
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
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
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

save timediff as second entry for timetracking

---
---
---
| id | 74a|
|---|---|
| timestamp | Sat Oct 21 23:30:06 2023 +0200|
| last modified | Sat Oct 21 23:30:06 2023 +0200|
| tags | |
| timetrack | |
| parents | |
| children | |

morgen wandern gehen

---
---
---
| id | 91d|
|---|---|
| timestamp | Sat Oct 21 23:40:10 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | wichtig|
| timetrack | |
| parents | cake|
| children | |

summe im timetrack

---
---
---
| id | f37|
|---|---|
| timestamp | Tue Oct 24 16:56:29 2023 +0200|
| last modified | Wed Feb 28 00:46:55 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

deterministic serialization order

---
---
---
| id | t_last|
|---|---|
| timestamp | Mon Feb 26 13:09:19 2024 +0100|
| last modified | Mon Feb 26 13:09:19 2024 +0100|
| tags | |
| timetrack | |
| parents | |
| children | |

der zur zeit letzte eintrag

---
---
---
| id | f70|
|---|---|
| timestamp | Wed Feb 28 00:51:06 2024 +0100|
| last modified | Wed Feb 28 00:51:06 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

exclustion with ~ for ids, parents and children as well

---
---
---
| id | 5e4|
|---|---|
| timestamp | Wed Feb 28 00:53:11 2024 +0100|
| last modified | Thu Feb 29 18:01:19 2024 +0100|
| tags | done|
| timetrack | |
| parents | cake|
| children | |

pretty md on add cmd

---
---
---
| id | d27|
|---|---|
| timestamp | Wed Feb 28 11:28:04 2024 +0100|
| last modified | Wed Feb 28 11:28:04 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

deadline date?

---
---
---
| id | aol|
|---|---|
| timestamp | Thu Feb 29 18:02:30 2024 +0100|
| last modified | Thu Feb 29 18:02:30 2024 +0100|
| tags | |
| timetrack | |
| parents | |
| children | |

# test ost eron

---
---
---
| id | dfa|
|---|---|
| timestamp | Thu Feb 29 18:19:41 2024 +0100|
| last modified | Thu Feb 29 18:19:41 2024 +0100|
| tags | |
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
| last modified | Thu Feb 29 18:19:59 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

stress test for big todo files

---
---
---
| id | ist|
|---|---|
| timestamp | Mon Mar  4 14:40:04 2024 +0100|
| last modified | Mon Mar  4 14:40:04 2024 +0100|
| tags | |
| timetrack | |
| parents | |
| children | |

do my assignment for ccc

---
---
---
| id | urd|
|---|---|
| timestamp | Wed Mar  6 11:33:40 2024 +0100|
| last modified | Wed Mar  6 11:33:40 2024 +0100|
| tags | |
| timetrack | |
| parents | cake|
| children | |

limit chars for long lines in short ls

---
---
---
| id | ntj|
|---|---|
| timestamp | Sat Mar  9 12:44:07 2024 +0100|
| last modified | Tue Mar 12 10:29:44 2024 +0100|
| tags | |
| timetrack | |
| parents | frech|
| children | tlv|

iwas

---
---
---
| id | tlv|
|---|---|
| timestamp | Tue Mar 12 10:29:44 2024 +0100|
| last modified | Tue Mar 12 10:29:44 2024 +0100|
| tags | |
| timetrack | |
| parents | ntj|
| children | |

child whatever
