//! the tree every module shows: the items a module works on (matches),
//! their parents up to `depth` levels as context, items changed this run
//! (kept until quit), and what the user opened (body and hidden children).

use super::*;
use std::collections::{HashMap, HashSet};

pub(super) struct Tree<'a> {
    all: &'a [Value],
    /// the module's items, editable
    pub matched: HashSet<String>,
    /// changed this run: shown with a check and editable, even when they no
    /// longer match
    pub changed: HashSet<String>,
    /// open rows: body under the line and every child shown
    pub opened: HashSet<String>,
    /// parents shown above a match (config tree_depth)
    pub depth: usize,
    /// a mark and color for open module items (answer: "? ")
    pub accent: Option<(&'static str, Color)>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Row {
    pub id: String,
    /// tree lines before the title
    pub prefix: String,
    /// tree lines before the lines under the title (body)
    pub below: String,
    /// a match or changed; the rest is context
    pub editable: bool,
    /// ancestors above this row are not shown
    pub cut_above: bool,
    /// children not shown
    pub hidden: usize,
    pub changed: bool,
    pub opened: bool,
}

impl<'a> Tree<'a> {
    pub fn new(all: &'a [Value], matched: impl IntoIterator<Item = String>, depth: usize) -> Self {
        Tree { all, matched: matched.into_iter().collect(), changed: HashSet::new(), opened: HashSet::new(), depth, accent: None }
    }

    pub fn item(&self, id: &str) -> Option<&'a Value> {
        find(self.all, id)
    }

    /// rows in tree order (the order of `all`, children under their parent)
    pub fn rows(&self) -> Vec<Row> {
        let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
        for i in self.all {
            children.entry(str_of(i, "parent")).or_default().push(str_of(i, "id"));
        }
        let exists = |id: &str| !id.is_empty() && self.item(id).is_some();
        let visible: HashSet<&str> = self.matched.iter().chain(&self.changed).map(String::as_str).filter(|id| exists(id)).collect();
        let mut shown: HashSet<&str> = HashSet::new();
        for v in &visible {
            shown.insert(v);
            let mut cur = str_of(self.item(v).unwrap(), "parent");
            for _ in 0..self.depth {
                if !exists(cur) {
                    break;
                }
                shown.insert(cur);
                cur = str_of(self.item(cur).unwrap(), "parent");
            }
        }
        // opened rows show all children; a shown child may be open as well
        loop {
            let more: Vec<&str> = shown
                .iter()
                .filter(|id| self.opened.contains(**id))
                .flat_map(|id| children.get(*id).into_iter().flatten().copied())
                .filter(|c| !shown.contains(c))
                .collect();
            if more.is_empty() {
                break;
            }
            shown.extend(more);
        }
        let mut out = vec![];
        let roots: Vec<&str> = self.all.iter().map(|i| str_of(i, "id")).filter(|id| shown.contains(id) && !shown.contains(str_of(self.item(id).unwrap(), "parent"))).collect();
        for r in roots {
            self.walk(r, ("", ""), &children, &shown, &visible, &mut out);
        }
        out
    }

    fn walk(&self, id: &str, prefix: (&str, &str), children: &HashMap<&str, Vec<&str>>, shown: &HashSet<&str>, visible: &HashSet<&str>, out: &mut Vec<Row>) {
        let g = style::g();
        let all_kids = children.get(id).map(Vec::as_slice).unwrap_or(&[]);
        let kids: Vec<&str> = all_kids.iter().copied().filter(|c| shown.contains(c)).collect();
        out.push(Row {
            id: id.to_string(),
            prefix: prefix.0.to_string(),
            below: prefix.1.to_string(),
            editable: visible.contains(id),
            cut_above: prefix.0.is_empty() && find(self.all, str_of(self.item(id).unwrap(), "parent")).is_some(),
            hidden: all_kids.len() - kids.len(),
            changed: self.changed.contains(id),
            opened: self.opened.contains(id),
        });
        for (n, c) in kids.iter().enumerate() {
            let last = n + 1 == kids.len();
            let own = format!("{}{}", prefix.1, if last { g.last } else { g.branch });
            let below = format!("{}{}", prefix.1, if last { g.space } else { g.pipe });
            self.walk(c, (&own, &below), children, shown, visible, out);
        }
    }

    /// the lines of `rows` at `width`, the cursor on row `cur`; `cols` gives
    /// the right-hand text of an item (status, who), `under` a line under
    /// the cursor row (the edit line); returns the lines and the line of
    /// the cursor
    pub fn lines(&self, rows: &[Row], cur: usize, width: u16, cols: &dyn Fn(&Value) -> String, under: Option<Vec<Span<'static>>>) -> (Vec<Line<'static>>, usize) {
        let (t, g) = (theme::t(), style::g());
        let w = width as usize;
        let mut out = vec![];
        let mut focus = 0;
        for (n, r) in rows.iter().enumerate() {
            let Some(item) = self.item(&r.id) else { continue };
            let here = n == cur;
            if here {
                focus = out.len();
            }
            let right = cols(item);
            let mut title = String::new();
            if r.changed {
                title.push_str(&format!("{} ", g.check));
            }
            if r.cut_above {
                title.push_str(&format!("{} {} ", g.ellipsis, g.select));
            }
            let accent = self.accent.filter(|_| r.editable && !r.changed);
            if let Some((mark, _)) = accent {
                title.push_str(mark);
            }
            title.push_str(str_of(item, "title"));
            if r.hidden > 0 && !r.opened {
                title.push_str(&format!(" +{}", r.hidden));
            }
            let lead = format!("{} {}", if here { g.select } else { " " }, r.prefix);
            let used = lead.chars().count() + right.chars().count() + 1;
            let title = cut(&title, w.saturating_sub(used).max(8));
            let style = match (here, r.editable, accent) {
                (true, _, Some((_, c))) => bold().fg(c),
                (true, _, None) => bold().fg(t.hint),
                (false, true, Some((_, c))) => Style::default().fg(c),
                (false, true, None) => Style::default(),
                (false, false, _) => dim(),
            };
            out.push(Line::from(vec![
                Span::styled(lead, if here { bold().fg(t.hint) } else { dim() }),
                Span::styled(title, style),
                Span::styled(format!(" {}", right), dim()),
            ]));
            // under the title: the tree lines go on, then the text
            let kids_below = rows.get(n + 1).is_some_and(|x| x.prefix.len() > r.prefix.len());
            let lead = format!("  {}{}", r.below, if kids_below { g.pipe } else { g.space });
            if let (true, Some(spans)) = (here, &under) {
                let mut line = vec![Span::styled(lead.clone(), dim())];
                line.extend(spans.iter().cloned());
                out.push(Line::from(line));
            }
            if r.opened {
                let body = body_of(item);
                let body = if body.is_empty() { "(no text)".to_string() } else { body };
                for l in markdown::render(&body, width.saturating_sub(lead.chars().count() as u16)) {
                    let mut spans = vec![Span::styled(lead.clone(), dim())];
                    spans.extend(l.spans.into_iter().map(|s| Span::styled(s.content.to_string(), s.style.patch(dim()))));
                    out.push(Line::from(spans));
                }
            }
        }
        (out, focus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(id: &str, parent: &str) -> Value {
        json!({"id": id, "title": format!("title {}", id), "parent": if parent.is_empty() { Value::Null } else { json!(parent) }, "body": format!("# title {}\n\nbody of {}", id, id)})
    }

    /// a > b > c > {d, e}; f top level
    fn all() -> Vec<Value> {
        vec![item("a", ""), item("b", "a"), item("c", "b"), item("d", "c"), item("e", "c"), item("f", "")]
    }

    fn ids(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.id.as_str()).collect()
    }

    #[test]
    fn parents_up_to_depth_and_a_cut_above() {
        let all = all();
        let tree = Tree::new(&all, ["d".to_string()], 2);
        let rows = tree.rows();
        assert_eq!(ids(&rows), ["b", "c", "d"]);
        assert!(rows[0].cut_above, "a is not shown");
        assert!(!rows[1].cut_above && !rows[2].cut_above);
        assert_eq!(rows.iter().map(|r| r.editable).collect::<Vec<_>>(), [false, false, true]);
        let tree = Tree::new(&all, ["d".to_string()], 0);
        assert_eq!(ids(&tree.rows()), ["d"]);
        assert!(tree.rows()[0].cut_above);
        let tree = Tree::new(&all, ["d".to_string(), "f".to_string()], 9);
        let rows = tree.rows();
        assert_eq!(ids(&rows), ["a", "b", "c", "d", "f"]);
        assert!(!rows[0].cut_above && !rows[4].cut_above);
    }

    #[test]
    fn hidden_children_counted_and_open_reveals_them() {
        let all = all();
        let mut tree = Tree::new(&all, ["d".to_string()], 1);
        let rows = tree.rows();
        assert_eq!(ids(&rows), ["c", "d"]);
        assert_eq!(rows[0].hidden, 1, "e is not shown");
        tree.opened.insert("c".into());
        let rows = tree.rows();
        assert_eq!(ids(&rows), ["c", "d", "e"]);
        assert_eq!(rows[0].hidden, 0);
        assert!(!rows[2].editable, "a revealed sibling is context");
        // opening a revealed row reveals its children too
        let mut tree = Tree::new(&all, ["c".to_string()], 0);
        tree.opened.extend(["c".to_string(), "d".to_string()]);
        assert_eq!(ids(&tree.rows()), ["c", "d", "e"]);
    }

    #[test]
    fn changed_items_stay_and_unknown_ids_are_skipped() {
        let all = all();
        let mut tree = Tree::new(&all, ["gone".to_string()], 2);
        assert!(tree.rows().is_empty());
        tree.changed.insert("f".into());
        let rows = tree.rows();
        assert_eq!(ids(&rows), ["f"]);
        assert!(rows[0].changed && rows[0].editable);
    }

    #[test]
    fn lines_show_marks_and_the_open_body() {
        style::init("unicode");
        let all = all();
        let mut tree = Tree::new(&all, ["d".to_string(), "e".to_string()], 1);
        tree.changed.insert("e".into());
        tree.opened.insert("d".into());
        let rows = tree.rows();
        let (lines, focus) = tree.lines(&rows, 2, 60, &|i| str_of(i, "id").to_string(), None);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        println!("{}", text.join("\n"));
        assert!(text[0].contains("… › title c"), "{}", text[0]);
        assert!(text[1].contains("├─ title d") && text[1].trim_end().ends_with('d'));
        assert!(text[2].contains("body of d"), "body under its line");
        assert!(text[3].starts_with("› └─ ✓ title e"), "{}", text[3]);
        assert_eq!(focus, 3);
    }

    #[test]
    #[ignore]
    fn dump_screens_tree() {
        style::init(&std::env::var("CAKE_STYLE").unwrap_or("unicode".into()));
        let mut all = vec![
            item("form", ""),
            item("dash", "form"),
            item("sect", "dash"),
            item("pie", "dash"),
            item("gantt", "form"),
            item("goal", "form"),
            item("pass", ""),
        ];
        for (n, t) in [(0, "Form"), (1, "Dashboard: one screen for what runs"), (2, "dashboard should come after the form"), (4, "Gantt view of tasks"), (5, "Time per goal for a period"), (6, "renew passport")] {
            all[n]["title"] = json!(t);
            all[n]["body"] = json!(format!("# {}\n\n- why: one line of text\n- done when: it shows", t));
        }
        let mut tree = Tree::new(&all, ["sect", "gantt", "pass"].map(String::from), 2);
        tree.changed.insert("goal".into());
        tree.opened.insert("gantt".into());
        for width in [60u16, 100] {
            let inner = column_width(width, 100).saturating_sub(4);
            let rows = tree.rows();
            let (card, focus) = tree.lines(&rows, 3, inner, &|i| if str_of(i, "id") == "goal" { "task ai".into() } else { "inbox".into() }, Some(vec![Span::raw("inbox · who – · prio – · due – · deadline –")]));
            let screen = Screen { card_title: "sort on the tree · cake".into(), card, focus: Some(focus), ..Default::default() };
            super::super::screens::dump("tree", width, &screen, 1);
        }
    }
}
