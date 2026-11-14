//! modules on the tree. sort: inbox items and tasks without assignee.
//! Every key saves at once; items changed this run stay with a check
//! until the form quits, and `u` puts an item back to how it was.

use super::tree::Tree;
use super::*;
use std::collections::HashSet;

/// an item before this run's first change, for undo
#[derive(Clone)]
pub(super) struct Before {
    status: String,
    who: Option<String>,
    prio: Option<u64>,
    due: Option<String>,
    deadline: Option<String>,
}

fn before(i: &Value) -> Before {
    Before {
        status: i["status"].as_str().unwrap_or("note").to_string(),
        who: i["assignee"].as_str().map(String::from),
        prio: i["priority"].as_u64(),
        due: i["due"].as_str().map(String::from),
        deadline: i["deadline"].as_str().map(String::from),
    }
}

fn closed(i: &Value) -> bool {
    matches!(str_of(i, "status"), "done" | "dropped")
}

fn is_sort(i: &Value) -> bool {
    let s = str_of(i, "status");
    s == "inbox" || (s == "task" && i["assignee"].is_null())
}

/// open items and those changed this run (also when closed now)
fn load(cake: &Cake, changed: &HashSet<String>) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(cake.call(&["ls", "-l", "-A"])?.into_iter().filter(|i| !closed(i) || changed.contains(str_of(i, "id"))).collect())
}

pub(super) fn count(cake: &Cake) -> Result<usize, Box<dyn Error>> {
    Ok(cake.call(&["ls", "-l"])?.iter().filter(|i| is_sort(i)).count())
}

/// `status · who · prio · due · deadline` of an item, set values bright
fn edit_line(i: &Value, user: &str) -> Vec<Span<'static>> {
    let none = "–".to_string();
    let who = i["assignee"].as_str().map(|w| if w == user { "me".to_string() } else { w.to_string() });
    let fields = [
        ("", Some(i["status"].as_str().unwrap_or("note").to_string())),
        ("who ", who),
        ("prio ", i["priority"].as_u64().map(|p| p.to_string())),
        ("due ", i["due"].as_str().map(|d| super::table::short_date(&Some(d.to_string())))),
        ("deadline ", i["deadline"].as_str().map(|d| super::table::short_date(&Some(d.to_string())))),
    ];
    let mut spans = vec![];
    for (n, (name, value)) in fields.into_iter().enumerate() {
        if n > 0 {
            spans.push(Span::styled(style::g().sep, dim()));
        }
        spans.push(Span::styled(name.to_string(), dim()));
        match value {
            Some(v) => spans.push(Span::styled(v, Style::default().fg(theme::t().hint))),
            None => spans.push(Span::styled(none.clone(), dim())),
        }
    }
    spans
}

/// one change to an item, through cake calls
enum Change {
    Status(&'static str),
    Who(Option<String>),
    Prio(Option<u64>),
    Due(Option<String>),
    Deadline(Option<String>),
    Move(String),
    Undo(Before),
}

/// a field set on an inbox item makes it a task first, with the default who
fn apply(cake: &Cake, item: &Value, change: &Change, sort_who: Option<&str>) -> Result<(), Box<dyn Error>> {
    let id = str_of(item, "id");
    let to_task = |cake: &Cake| -> Result<(), Box<dyn Error>> {
        if str_of(item, "status") == "inbox" {
            cake.call(&["set", "task", id])?;
            if let (true, Some(w)) = (item["assignee"].is_null(), sort_who) {
                cake.call(&["edit", id, "--assignee", w])?;
            }
        }
        Ok(())
    };
    let opt = |v: &Option<String>| v.clone().unwrap_or("-".into());
    match change {
        Change::Status("task") => to_task(cake)?,
        Change::Status(s) => {
            cake.call(&["set", s, id])?;
        }
        Change::Who(w) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--assignee", &opt(w)])?;
        }
        Change::Prio(p) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--prio", &p.unwrap_or(0).to_string()])?;
        }
        Change::Due(d) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--due", &opt(d)])?;
        }
        Change::Deadline(d) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--deadline", &opt(d)])?;
        }
        Change::Move(to) => {
            cake.call(&["mv", id, "--to-store", to])?;
        }
        Change::Undo(b) => {
            cake.call(&["set", &b.status, id])?;
            let (prio, due, deadline) = (b.prio.unwrap_or(0).to_string(), opt(&b.due), opt(&b.deadline));
            cake.call(&["edit", id, "--assignee", &opt(&b.who), "--prio", &prio, "--due", &due, "--deadline", &deadline])?;
        }
    }
    Ok(())
}

/// runs sort on the tree in one store; returns (items changed, how it left)
pub(super) fn run(ui: &mut Ui, cake: &Cake, root: &Path, user: &str, store: &str) -> Result<(usize, Leave), Box<dyn Error>> {
    let sort_who = match ui.sort_who.as_str() {
        "-" => None,
        "me" => Some(user.to_string()),
        w => Some(w.to_string()),
    };
    let stores: Vec<String> = crate::stores::all()?.into_iter().map(|(n, _)| n).filter(|n| n != store).collect();
    let whos = super::table::who_cycle(root, user);
    let mut opened: HashSet<String> = HashSet::new();
    let mut cur_id = String::new();
    let (mut help, mut note) = (false, None::<(String, bool)>);
    let mut decided = 0;
    let page = 10usize;
    loop {
        let changed = ui.changed.entry(root.to_path_buf()).or_default().clone();
        let all = load(cake, &changed)?;
        let mut tree = Tree::new(&all, all.iter().filter(|i| is_sort(i)).map(|i| str_of(i, "id").to_string()), ui.tree_depth);
        tree.changed = changed;
        tree.opened = opened.clone();
        let rows = tree.rows();
        if rows.is_empty() {
            return Ok((decided, Leave::Empty));
        }
        // the cursor stays on its item; at first the first editable row
        let cur = rows.iter().position(|r| r.id == cur_id).or_else(|| rows.iter().position(|r| r.editable)).unwrap_or(0);
        cur_id = rows[cur].id.clone();
        let item = tree.item(&cur_id).cloned().unwrap_or_default();
        let editable = rows[cur].editable;
        let width = column_width(ui.term.size()?.width, ui.max_width).saturating_sub(4);
        let who_of = |i: &Value| i["assignee"].as_str().map(|w| if w == user { "me" } else { w }).unwrap_or("").to_string();
        let cols = |i: &Value| format!("{} {}", i["status"].as_str().unwrap_or("note"), who_of(i)).trim_end().to_string();
        let under = editable.then(|| edit_line(&item, user));
        let (lines, focus) = tree.lines(&rows, cur, width, &cols, under);
        let k = &ui.keys;
        let ks = |a: Act| k.keys_of(a);
        let parts = [
            (format!("{}/{}/{}", ks(Act::Task), ks(Act::Note), ks(Act::Drop)), "task note drop"),
            (ks(Act::Who), "who"),
            (ks(Act::Prio), "prio"),
            (":due :deadline :move".to_string(), ""),
            (ks(Act::Undo), "undo"),
            (ks(Act::Open), "open"),
            (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module"),
            (ks(Act::Help), "keys"),
        ];
        let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), *b)).collect();
        let mut screen = Screen {
            card_title: format!("inbox and tasks without assignee · {}", store),
            card: lines,
            focus: Some(focus),
            keys: keys(&parts),
            note: note.take(),
            help: if help { ui.keys.help() } else { vec![] },
            ..Default::default()
        };
        ui.draw(&screen)?;
        let act = ui.keys.act()?;
        if help {
            help = false;
            continue;
        }
        let last = rows.len() - 1;
        let step = |from: usize, down: bool| -> usize {
            let mut n = from;
            loop {
                n = if down { n + 1 } else { n.wrapping_sub(1) };
                match rows.get(n) {
                    Some(r) if r.editable => return n,
                    Some(_) => (),
                    None => return from,
                }
            }
        };
        let mut change = None;
        let mut then_next = false;
        let goto = |n: usize| rows[n].id.clone();
        match act {
            Act::Down => cur_id = goto((cur + 1).min(last)),
            Act::Up => cur_id = goto(cur.saturating_sub(1)),
            Act::Top => cur_id = goto(0),
            Act::Bottom => cur_id = goto(last),
            Act::HalfDown => cur_id = goto((cur + page).min(last)),
            Act::HalfUp => cur_id = goto(cur.saturating_sub(page)),
            Act::Next => cur_id = goto(step(cur, true)),
            Act::Previous => cur_id = goto(step(cur, false)),
            Act::Help => help = true,
            Act::Open => {
                if !opened.remove(&cur_id) {
                    opened.insert(cur_id.clone());
                }
            }
            Act::Back => opened.clear(),
            Act::PrevModule => return Ok((decided, Leave::Prev)),
            Act::NextModule => return Ok((decided, Leave::Next)),
            Act::Confirm => return Ok((decided, Leave::Done)),
            Act::Quit => return Ok((decided, Leave::Quit)),
            Act::Feedback => {
                feedback(ui, &mut screen, cake, store, &format!("sort tree, row {}", cur_id))?;
                note = screen.note.take();
            }
            Act::Task | Act::Note | Act::Drop | Act::Who | Act::Prio | Act::Undo | Act::Subtree | Act::Command if !editable => {
                note = Some(("a parent for context: not in sort (o opens it)".into(), true));
            }
            Act::Task => change = Some(Change::Status("task")),
            Act::Note => (change, then_next) = (Some(Change::Status("note")), true),
            Act::Drop => (change, then_next) = (Some(Change::Status("dropped")), true),
            Act::Who => {
                let at = item["assignee"].as_str().and_then(|w| whos.iter().position(|x| x == w));
                change = Some(Change::Who(Some(whos[at.map_or(0, |a| (a + 1) % whos.len())].clone())));
            }
            Act::Prio => {
                change = Some(Change::Prio(match item["priority"].as_u64() {
                    None => Some(1),
                    Some(p) if p >= 3 => None,
                    Some(p) => Some(p + 1),
                }))
            }
            Act::Undo => match ui.before.get(&(root.to_path_buf(), cur_id.clone())) {
                Some(b) => change = Some(Change::Undo(b.clone())),
                None => note = Some(("nothing to undo here".into(), false)),
            },
            Act::Subtree => {
                // this row's status and who for every sort item below it
                let below: Vec<Value> = all
                    .iter()
                    .filter(|i| is_sort(i) && str_of(i, "id") != cur_id && {
                        let mut p = find(&all, str_of(i, "parent"));
                        let mut hit = false;
                        while let Some(x) = p {
                            if str_of(x, "id") == cur_id {
                                hit = true;
                                break;
                            }
                            p = find(&all, str_of(x, "parent"));
                        }
                        hit
                    })
                    .cloned()
                    .collect();
                let status: &'static str = match str_of(&item, "status") {
                    "task" => "task",
                    "dropped" => "dropped",
                    "" | "note" => "note",
                    _ => "inbox",
                };
                let mut n = 0;
                for b in &below {
                    let mut changes = vec![Change::Status(status)];
                    if status == "task" {
                        changes.push(Change::Who(item["assignee"].as_str().map(String::from)));
                    }
                    for c in &changes {
                        if let Err(e) = save_one(ui, &mut screen, cake, root, b, c, sort_who.as_deref(), &mut decided)? {
                            note = Some((e, true));
                        }
                    }
                    n += 1;
                }
                if note.is_none() {
                    note = Some((format!("{} items below took this row's status", n), false));
                }
            }
            Act::Command => {
                let Some(line) = ui.input(&mut screen, ":")? else { continue };
                let (cmd, arg) = line.split_once(' ').map_or((line.as_str(), ""), |(c, a)| (c, a.trim()));
                let date = |a: &str| match a {
                    "" | "-" => Ok(None),
                    a => crate::util::parse_date(a).map(Some),
                };
                match cmd {
                    "q" | "q!" | "wq" | "x" => return Ok((decided, Leave::Quit)),
                    "w" => return Ok((decided, Leave::Done)),
                    "due" => match date(arg) {
                        Ok(v) => change = Some(Change::Due(v)),
                        Err(e) => note = Some((e, true)),
                    },
                    "deadline" => match date(arg) {
                        Ok(v) => change = Some(Change::Deadline(v)),
                        Err(e) => note = Some((e, true)),
                    },
                    "who" => match crate::config::Config::default().who(root, if arg.is_empty() { "-" } else { arg }) {
                        Ok(w) => change = Some(Change::Who(if arg == "me" { Some(user.to_string()) } else { w })),
                        Err(e) => note = Some((e.to_string(), true)),
                    },
                    "prio" => match arg.parse::<u64>() {
                        Ok(p) if (1..=3).contains(&p) => change = Some(Change::Prio(Some(p))),
                        _ if arg.is_empty() || arg == "-" || arg == "0" => change = Some(Change::Prio(None)),
                        _ => note = Some(("prio is 1, 2, 3 or - for none".into(), true)),
                    },
                    "move" if stores.iter().any(|s| s == arg) => change = Some(Change::Move(arg.to_string())),
                    "move" => note = Some((format!("no store '{}': {}", arg, stores.join(", ")), true)),
                    other => note = Some((format!("unknown command :{}", other), true)),
                }
            }
            _ => (),
        }
        if let Some(c) = change {
            if let Err(e) = save_one(ui, &mut screen, cake, root, &item, &c, sort_who.as_deref(), &mut decided)? {
                note = Some((e, true));
            } else if then_next {
                cur_id = goto(step(cur, true));
            }
        }
    }
}

/// applies one change, remembers the item as changed and how it was
#[allow(clippy::too_many_arguments)]
fn save_one(
    ui: &mut Ui,
    screen: &mut Screen,
    cake: &Cake,
    root: &Path,
    item: &Value,
    change: &Change,
    sort_who: Option<&str>,
    decided: &mut usize,
) -> Result<Result<(), String>, Box<dyn Error>> {
    let id = str_of(item, "id").to_string();
    let key = (root.to_path_buf(), id.clone());
    let result = ui.save(screen, || apply(cake, item, change, sort_who))?;
    if let Err(e) = result {
        return Ok(Err(format!("{}: {}", id, e)));
    }
    let changed = ui.changed.entry(root.to_path_buf()).or_default();
    if let Change::Undo(_) = change {
        changed.remove(&id);
        ui.before.remove(&key);
        *decided = decided.saturating_sub(1);
        ui.handled = ui.handled.saturating_sub(1);
    } else if !ui.before.contains_key(&key) {
        changed.insert(id);
        ui.before.insert(key, before(item));
        *decided += 1;
        ui.handled += 1;
    }
    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn edit_line_shows_values_and_dashes() {
        style::init("unicode");
        let i = json!({"id": "a", "status": "task", "assignee": "jul", "priority": 2, "due": "2026-11-20"});
        let line = Line::from(edit_line(&i, "jul")).to_string();
        assert_eq!(line, "task · who me · prio 2 · due 20.11. · deadline –");
        let i = json!({"id": "a", "status": "inbox"});
        assert_eq!(Line::from(edit_line(&i, "jul")).to_string(), "inbox · who – · prio – · due – · deadline –");
    }
}
