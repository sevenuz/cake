[//]: # (cake save file)
[//]: # ({"last_write": 1234, "save_file": "md"})

---
###### id0
# Beispiel Markdown save file...
---
###### id1 [tag1, tag2] Parents: parent_id1, parent_id2 Children: children_id1, children_id2

# Here ist der Content dieses Todos, welches markdown nutzt
- das und hier
- jenes und weiteres
Noch ein letzer Satz
Hier genutzte Delimiter müssen escaped werden...

\---

\***

\___

---
###### id2 [tag1, tag4]
Dieser Todo hat keine parents oder children und ist nur einfacher Text.

---
###### id3 Parents: [id1](#id1-tag1-tag2-parents-parent_id1-parent_id2-children-children_id1-children_id2)
Die 6er Überschrift wird für Meta genutzt, unwahrscheinlich dass man danach gleich wieder ne 6er nutzt...

Parents und Children sollten zu links gemacht werden:


    The IDs are generated from the content of the header according to the following rules:

        All text is converted to lowercase.
        All non-word text (e.g., punctuation, HTML) is removed.
        All spaces are converted to hyphens.
        Two or more hyphens in a row are converted to one.
        If a header with the same ID has already been generated, a unique incrementing number is appended, starting at 1.



---
