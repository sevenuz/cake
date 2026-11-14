//! every module on the tree: news (what agents did), inbox (new items and
//! tasks without assignee), answer (questions for you), review (finished
//! work you own), mine (your tasks). The tree fills the screen; the
//! module's items are editable, parents give context. Every key saves at
//! once; items changed this run stay with a check until the form quits,
//! and `u` puts an item back to how it was.

use super::tree::Tree;
use super::*;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Kind {
    News,
    Sort,
    Answer,
    Review,
    Mine,
}

impl Kind {
    pub fn of(name: &str) -> Option<Kind> {
        Some(match name {
            "news" => Kind::News,
            "inbox" => Kind::Sort,
            "answer" => Kind::Answer,
            "review" => Kind::Review,
            "mine" => Kind::Mine,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Kind::News => "news",
            Kind::Sort => "inbox",
            Kind::Answer => "answer",
            Kind::Review => "review",
            Kind::Mine => "mine",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Kind::News => "what agents did since your last run",
            Kind::Sort => "inbox: new items and tasks without assignee",
            Kind::Answer => "questions for you",
            Kind::Review => "finished work to check",
            Kind::Mine => "your tasks",
        }
    }

    /// modules where the edit line shows and fields can be set
    fn edits(self) -> bool {
        matches!(self, Kind::Sort | Kind::Mine | Kind::News)
    }

    /// the mark (None: status mark) and color of the module's open items
    fn accent(self) -> (Option<&'static str>, Color) {
        let t = theme::t();
        match self {
            Kind::News => (None, t.hint),
            Kind::Sort => (None, t.heading),
            Kind::Answer => (Some("?"), t.question),
            Kind::Review => (None, t.code),
            Kind::Mine => (None, t.doing),
        }
    }

    /// modules with a card under the tree
    fn cards(self) -> bool {
        matches!(self, Kind::Answer | Kind::Review)
    }
}

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

fn ts(v: &Value) -> i64 {
    v.as_str().and_then(|t| crate::util::parse_timestamp(t).ok()).unwrap_or(0)
}

/// news: new, started or finished by an agent since the last run
fn news_what(i: &Value, user: &str, since: i64) -> Option<&'static str> {
    let agent = !i["by"].is_null() && str_of(i, "by") != user;
    if since == 0 || !agent {
        return None;
    }
    let started = i["time"].as_array().is_some_and(|t| t.iter().any(|t| ts(&t["start"]) >= since));
    if ts(&i["created"]) >= since {
        Some("new")
    } else if str_of(i, "status") == "review" && started {
        Some("finished")
    } else if started {
        Some("started")
    } else {
        None
    }
}

/// is `i` one of the module's items
fn matches(kind: Kind, i: &Value, all: &[Value], user: &str, since: i64) -> bool {
    let status = str_of(i, "status");
    let open = |id: &str| find(all, id).is_some_and(|x| matches!(str_of(x, "status"), "inbox" | "task" | "doing" | "review"));
    let blocks_open = i["blocks"].as_array().is_some_and(|b| b.iter().any(|b| open(b.as_str().unwrap_or(""))));
    match kind {
        Kind::News => news_what(i, user, since).is_some(),
        Kind::Sort => status == "inbox" || (status == "task" && i["assignee"].is_null()),
        // a task for you that someone else's work waits on
        Kind::Answer => status == "task" && str_of(i, "assignee") == user && blocks_open,
        Kind::Review => status == "review" && (str_of(i, "owner") == user || i["owner"].is_null()),
        Kind::Mine => str_of(i, "assignee") == user && matches!(status, "task" | "doing") && !blocks_open,
    }
}

/// all items with closed ones only when changed this run
fn load(cake: &Cake, changed: &HashSet<String>) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(cake.call(&["ls", "-l", "-A"])?.into_iter().filter(|i| !closed(i) || changed.contains(str_of(i, "id"))).collect())
}

pub(super) fn count(cake: &Cake, kind: Kind, user: &str, since: i64) -> Result<usize, Box<dyn Error>> {
    let all = cake.call(&["ls", "-l"])?;
    Ok(all.iter().filter(|i| matches(kind, i, &all, user, since)).count())
}

fn short_date(d: &str) -> String {
    chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d").map_or(d.to_string(), |d| d.format("%d.%m.").to_string())
}

fn who_of(i: &Value, user: &str) -> Option<String> {
    i["assignee"].as_str().map(|w| if w == user { "me".to_string() } else { w.to_string() })
}

/// `status · who · prio · due · deadline` of an item, set values bright
fn edit_line(i: &Value, user: &str) -> Vec<Span<'static>> {
    let fields = [
        ("", Some(i["status"].as_str().unwrap_or("note").to_string())),
        ("who ", who_of(i, user)),
        ("prio ", i["priority"].as_u64().map(|p| p.to_string())),
        ("due ", i["due"].as_str().map(short_date)),
        ("deadline ", i["deadline"].as_str().map(short_date)),
    ];
    let mut spans = vec![];
    for (n, (name, value)) in fields.into_iter().enumerate() {
        if n > 0 {
            spans.push(Span::styled(style::g().sep, dim()));
        }
        spans.push(Span::styled(name.to_string(), dim()));
        match value {
            Some(v) => spans.push(Span::styled(v, Style::default().fg(theme::t().hint))),
            None => spans.push(Span::styled("–", dim())),
        }
    }
    spans
}

/// who `w` cycles through: you, ai, agents and people of the store config
fn who_cycle(root: &Path, user: &str) -> Vec<String> {
    let sc = crate::config::Config::store(root).unwrap_or_default();
    let mut list = vec![user.to_string(), "ai".to_string()];
    for a in sc.agents.iter().chain(&sc.people) {
        if !list.contains(a) {
            list.push(a.clone());
        }
    }
    list
}

/// the card of an answer or review item: text and options (the last one
/// takes own words)
fn card_of(kind: Kind, item: &Value, all: &[Value]) -> (String, Vec<String>) {
    let g = style::g();
    let body = body_of(item);
    let (context, mut options): (String, Vec<String>) = match kind {
        Kind::Answer => {
            let (c, o) = split_options(&body);
            (c, o.into_iter().map(|(_, t)| t).collect())
        }
        _ => {
            // a parent: its parts with their result
            let parts: Vec<String> = all
                .iter()
                .filter(|c| str_of(c, "parent") == str_of(item, "id") && !c["status"].is_null())
                .map(|c| format!("- {} {}", if str_of(c, "status") == "done" { g.check } else { g.dropped }, str_of(c, "title")))
                .collect();
            let text = if parts.is_empty() { body } else { format!("{}\n\nparts:\n{}", body, parts.join("\n")) };
            (text, vec!["ok, done".into(), "drop".into()])
        }
    };
    options.push(format!("{} {}", g.ellipsis, if kind == Kind::Answer { "own words" } else { "change something" }));
    (context, options)
}

/// one change to an item, through cake calls
enum Change {
    Status(&'static str),
    Who(Option<String>),
    Prio(Option<u64>),
    Due(Option<String>),
    Deadline(Option<String>),
    Move(String),
    Timer(bool),
    /// news: back to you; a started item waits as a task
    Hold,
    /// answer: an option's text or own words
    Answer(String),
    /// review: what should change
    Comment(String),
    Undo(Before),
}

/// a field set on an inbox item makes it a task first, with the default who
fn apply(cake: &Cake, item: &Value, change: &Change, user: &str, sort_who: Option<&str>) -> Result<String, Box<dyn Error>> {
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
    Ok(match change {
        Change::Status("task") => {
            to_task(cake)?;
            String::new()
        }
        Change::Status(s) => {
            cake.call(&["set", s, id])?;
            String::new()
        }
        Change::Who(w) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--assignee", &opt(w)])?;
            String::new()
        }
        Change::Prio(p) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--prio", &p.unwrap_or(0).to_string()])?;
            String::new()
        }
        Change::Due(d) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--due", &opt(d)])?;
            String::new()
        }
        Change::Deadline(d) => {
            to_task(cake)?;
            cake.call(&["edit", id, "--deadline", &opt(d)])?;
            String::new()
        }
        Change::Move(to) => {
            cake.call(&["mv", id, "--to-store", to])?;
            format!("moved to {}", to)
        }
        Change::Timer(start) => {
            cake.call(&[if *start { "start" } else { "stop" }, id])?;
            String::new()
        }
        Change::Hold => {
            cake.call(&["edit", id, "--assignee", user])?;
            if str_of(item, "status") == "doing" {
                cake.call(&["set", "task", id])?;
            }
            "held: back to you".into()
        }
        Change::Answer(text) => {
            cake.call(&["set", "done", id, "-m", &format!("answer: {}", text)])?;
            text.clone()
        }
        Change::Comment(text) => {
            cake.call(&["edit", id, "--comment", text])?;
            "sent back".into()
        }
        Change::Undo(b) => {
            cake.call(&["set", &b.status, id])?;
            let (prio, due, deadline) = (b.prio.unwrap_or(0).to_string(), opt(&b.due), opt(&b.deadline));
            cake.call(&["edit", id, "--assignee", &opt(&b.who), "--prio", &prio, "--due", &due, "--deadline", &deadline])?;
            String::new()
        }
    })
}

/// a column right of the tree
#[derive(Clone, Copy, PartialEq, Debug)]
enum Col {
    What,
    Status,
    Who,
    Prio,
    Due,
    Deadline,
}

impl Col {
    fn title(self) -> &'static str {
        match self {
            Col::What => "what",
            Col::Status => "status",
            Col::Who => "who",
            Col::Prio => "prio",
            Col::Due => "due",
            Col::Deadline => "deadline",
        }
    }

    fn width(self) -> usize {
        match self {
            Col::What => 8,
            Col::Status => 7,
            Col::Who => 11,
            Col::Prio => 4,
            Col::Due => 6,
            Col::Deadline => 8,
        }
    }
}

/// the columns of a module at `width`; narrow: no prio and deadline
fn columns(kind: Kind, width: u16) -> Vec<Col> {
    let mut c = vec![];
    if kind == Kind::News {
        c.push(Col::What);
    }
    c.extend([Col::Status, Col::Who]);
    if width >= 80 {
        c.extend([Col::Prio, Col::Due, Col::Deadline]);
    } else {
        c.push(Col::Due);
    }
    c
}

fn cols_width(cols: &[Col]) -> usize {
    cols.iter().map(|c| c.width() + 1).sum()
}

/// the column titles, right-aligned over the columns of `width`
fn cols_header(cols: &[Col], width: u16) -> Line<'static> {
    let mut spans = vec![Span::raw(" ".repeat((width as usize).saturating_sub(cols_width(cols))))];
    for c in cols {
        spans.push(Span::styled(format!("{} ", cut(c.title(), c.width())), dim().add_modifier(Modifier::UNDERLINED)));
    }
    Line::from(spans)
}

/// the cells of an item: muted colors, red when late, all dim for context
fn col_spans(cols: &[Col], i: &Value, user: &str, since: i64, context: bool) -> Vec<Span<'static>> {
    let t = theme::t();
    let today = chrono::Local::now().date_naive();
    let date = |d: Option<&str>| -> (String, Style) {
        let Some(d) = d else { return ("–".into(), dim()) };
        let style = match chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d") {
            Ok(x) if x < today => Style::default().fg(t.error),
            Ok(x) if (x - today).num_days() <= 2 => Style::default().fg(t.doing),
            _ => Style::default(),
        };
        (short_date(d), style)
    };
    let mut out = vec![];
    for c in cols {
        let (text, style) = match c {
            Col::What => (news_what(i, user, since).unwrap_or("").to_string(), Style::default().fg(t.hint)),
            Col::Status => {
                let s = i["status"].as_str().unwrap_or("note");
                let style = match s {
                    "inbox" => Style::default().fg(t.hint),
                    "doing" => Style::default().fg(t.doing),
                    "review" => Style::default().fg(t.code),
                    "done" => Style::default().fg(t.ok),
                    "task" => Style::default(),
                    _ => dim(),
                };
                (if i["running"].as_bool() == Some(true) { "running".to_string() } else { s.to_string() }, style)
            }
            Col::Who => match who_of(i, user) {
                Some(w) if w == "me" => (w, Style::default()),
                Some(w) => (w, Style::default().fg(t.hint)),
                None => ("–".into(), dim()),
            },
            Col::Prio => match i["priority"].as_u64() {
                Some(1) => ("1".into(), Style::default().fg(t.error)),
                Some(2) => ("2".into(), Style::default().fg(t.doing)),
                Some(p) => (p.to_string(), Style::default()),
                None => ("–".into(), dim()),
            },
            Col::Due => date(i["due"].as_str()),
            Col::Deadline => date(i["deadline"].as_str()),
        };
        // muted: never bold; context rows all dim
        let style = if context { dim() } else { style.add_modifier(Modifier::DIM) };
        out.push(Span::styled(format!("{} ", cut(&text, c.width())), style));
    }
    out
}

/// what the module shows for one state; pure, so screens can be dumped
struct View<'a> {
    kind: Kind,
    user: &'a str,
    since: i64,
    /// results of this run per id (an answer, "sent back")
    results: &'a std::collections::HashMap<String, String>,
}

impl View<'_> {
    /// the detail box of the hovered row: title, the fields, the whole
    /// text; for an open answer or review item its options (`sel` marked)
    fn detail(&self, item: &Value, all: &[Value], actions: bool, sel: Option<usize>, width: u16) -> (String, Vec<Line<'static>>, Vec<Line<'static>>) {
        let (t, g) = (theme::t(), style::g());
        let with_options = actions && self.kind.cards() && matches(self.kind, item, all, self.user, self.since);
        let (text, options) = if with_options { card_of(self.kind, item, all) } else { (body_of(item), vec![]) };
        let mut lines = vec![Line::from(edit_line(item, self.user))];
        if let Some(what) = news_what(item, self.user, self.since).filter(|_| self.kind == Kind::News) {
            lines.push(Line::from(vec![Span::styled("what ", dim()), Span::raw(what)]));
        }
        if let Some(r) = self.results.get(str_of(item, "id")) {
            lines.push(Line::from(vec![Span::styled("this run ", dim()), Span::styled(r.clone(), Style::default().fg(t.ok))]));
        }
        if !text.trim().is_empty() {
            lines.push(Line::default());
            lines.extend(markdown::render(&text, width.saturating_sub(4)));
        }
        let choices = options
            .iter()
            .enumerate()
            .map(|(n, text)| {
                let chosen = sel == Some(n);
                Line::from(vec![
                    Span::styled(format!("{} {}) ", if chosen { g.select } else { " " }, n + 1), Style::default().fg(t.hint)),
                    Span::styled(text.clone(), if chosen { bold().fg(t.hint) } else { Style::default() }),
                ])
            })
            .collect();
        let title = if actions { str_of(item, "title").to_string() } else { format!("{} · context, no actions here", str_of(item, "title")) };
        (title, lines, choices)
    }
}

/// the state of one module run
struct State {
    cur_id: String,
    opened: HashSet<String>,
    /// the detail box has the focus
    in_box: bool,
    /// the option marked in the box (answer, review)
    sel: usize,
    /// lines the box text is scrolled
    scroll: u16,
    help: bool,
    note: Option<(String, bool)>,
    /// something was saved since the last draw (for form_auto_next)
    saved: bool,
}

/// runs one module in one store; returns (items changed, how it left)
pub(super) fn run(ui: &mut Ui, cake: &Cake, root: &Path, kind: Kind, user: &str, store: &str, since: i64) -> Result<(usize, Leave), Box<dyn Error>> {
    let sort_who = match ui.sort_who.as_str() {
        "-" => None,
        "me" => Some(user.to_string()),
        w => Some(w.to_string()),
    };
    let stores: Vec<String> = crate::stores::all()?.into_iter().map(|(n, _)| n).filter(|n| n != store).collect();
    let whos = who_cycle(root, user);
    let mut st = State { cur_id: String::new(), opened: HashSet::new(), in_box: false, sel: 0, scroll: 0, help: false, note: None, saved: false };
    let mut decided = 0;
    let page = 10usize;
    loop {
        let changed = ui.changed.entry(root.to_path_buf()).or_default().clone();
        let results: std::collections::HashMap<String, String> =
            ui.results.iter().filter(|((r, _), _)| r == root).map(|((_, id), v)| (id.clone(), v.clone())).collect();
        let mut all = load(cake, &changed)?;
        keep_order(ui.order.entry(root.to_path_buf()).or_default(), &mut all);
        let matched: Vec<String> = all.iter().filter(|i| matches(kind, i, &all, user, since)).map(|i| str_of(i, "id").to_string()).collect();
        let mut tree = Tree::new(&all, matched, ui.tree_depth);
        tree.changed = changed.clone();
        tree.opened = st.opened.clone();
        tree.accent = Some(kind.accent());
        let rows = tree.rows();
        if rows.is_empty() {
            return Ok((decided, Leave::Empty));
        }
        // the last open item done: on to the next module
        if std::mem::take(&mut st.saved) && ui.auto_next && !rows.iter().any(|r| r.editable && !r.changed) {
            return Ok((decided, Leave::Done));
        }
        // the cursor stays on its item; at first the first open module item
        let cur = rows
            .iter()
            .position(|r| r.id == st.cur_id)
            .or_else(|| rows.iter().position(|r| r.editable && !r.changed))
            .or_else(|| rows.iter().position(|r| r.editable))
            .unwrap_or(0);
        if rows[cur].id != st.cur_id {
            (st.sel, st.scroll) = (0, 0);
        }
        st.cur_id = rows[cur].id.clone();
        let item = tree.item(&st.cur_id).cloned().unwrap_or_default();
        let editable = rows[cur].editable;
        // answer, review: options for an open item
        let with_options = editable && kind.cards() && matches(kind, &item, &all, user, since);
        let n_opts = if with_options { card_of(kind, &item, &all).1.len() } else { 0 };
        let view = View { kind, user, since, results: &results };
        let width = column_width(ui.term.size()?.width, ui.max_width);
        let cols = columns(kind, width);
        let lines = tree.lines(&rows, cur, width, &|i, context| col_spans(&cols, i, user, since, context), cols_width(&cols), st.in_box);
        let (card_title, card, choices) = view.detail(&item, &all, editable, (st.in_box && with_options).then_some(st.sel), width);
        let k = &ui.keys;
        let ks = |a: Act| k.keys_of(a);
        let mut parts: Vec<(String, String)> = if st.in_box && with_options {
            vec![("j/k 1-9".into(), "choose".into()), (ks(Act::Confirm), "save".into()), (ks(Act::Insert), "own words".into())]
        } else if st.in_box {
            vec![("j/k".into(), "scroll".into())]
        } else {
            match kind {
                Kind::Sort => vec![
                    (format!("{}/{}/{}", ks(Act::Task), ks(Act::Note), ks(Act::Drop)), "task note drop".into()),
                    (ks(Act::Who), "who".into()),
                    (ks(Act::Prio), "prio".into()),
                    (":due".into(), String::new()),
                ],
                Kind::News => vec![(ks(Act::Who), "hold".into()), (ks(Act::Drop), "drop".into())],
                Kind::Mine => vec![
                    (format!("{}/{}/{}", ks(Act::Timer), ks(Act::Complete), ks(Act::Drop)), "start done drop".into()),
                    (ks(Act::Who), "who".into()),
                    (ks(Act::Prio), "prio".into()),
                ],
                Kind::Answer => vec![(format!("{}/{}", ks(Act::Note), ks(Act::Prio)), "question".into()), ("1-9".into(), "pick".into()), (ks(Act::Insert), "own words".into())],
                Kind::Review => vec![
                    (format!("{}/{}", ks(Act::Note), ks(Act::Prio)), "next prev".into()),
                    (ks(Act::Complete), "ok".into()),
                    (ks(Act::Drop), "drop".into()),
                    (ks(Act::Insert), "change".into()),
                ],
            }
        };
        if !st.in_box {
            parts.push((ks(Act::Open), "children".into()));
        }
        parts.extend([
            (ks(Act::Focus), if st.in_box { "tree".into() } else { "details".into() }),
            (ks(Act::Editor), "edit".into()),
            (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module".into()),
            (ks(Act::Help), "keys".into()),
        ]);
        let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let mut tree_lines = vec![Line::from(Span::styled(format!("{} · {}", kind.title(), store), bold().fg(kind.accent().1))), cols_header(&cols, width)];
        tree_lines.extend(lines);
        let mut screen = Screen {
            tree: tree_lines,
            tree_focus: Some(cur + 2),
            card_title,
            card,
            choices,
            card_active: st.in_box,
            scroll: st.scroll,
            keys: keys(&parts),
            note: st.note.take(),
            help: if st.help { ui.keys.help() } else { vec![] },
            ..Default::default()
        };
        ui.draw(&screen)?;
        if st.help {
            st.help = false;
            let _ = ui.keys.act()?;
            continue;
        }
        let act = ui.keys.act()?;
        let last = rows.len() - 1;
        // the next or previous open module item
        let step = |from: usize, down: bool| -> usize {
            let mut n = from;
            loop {
                n = if down { n + 1 } else { n.wrapping_sub(1) };
                match rows.get(n) {
                    Some(r) if r.editable && !r.changed => return n,
                    Some(_) => (),
                    None => return from,
                }
            }
        };
        let goto = |n: usize| rows[n].id.clone();

        // keys only the box has
        if st.in_box {
            let handled = match act {
                Act::Down if with_options => {
                    st.sel = (st.sel + 1).min(n_opts - 1);
                    true
                }
                Act::Up if with_options => {
                    st.sel = st.sel.saturating_sub(1);
                    true
                }
                Act::Down => {
                    st.scroll = st.scroll.saturating_add(1);
                    true
                }
                Act::Up => {
                    st.scroll = st.scroll.saturating_sub(1);
                    true
                }
                Act::HalfDown => {
                    st.scroll = st.scroll.saturating_add(5);
                    true
                }
                Act::HalfUp => {
                    st.scroll = st.scroll.saturating_sub(5);
                    true
                }
                Act::Back => {
                    st.in_box = false;
                    true
                }
                Act::Confirm if !with_options => true,
                _ => false,
            };
            if handled {
                continue;
            }
        }

        let mut change = None;
        let mut then_next = false;
        match act {
            Act::Down => st.cur_id = goto((cur + 1).min(last)),
            Act::Up => st.cur_id = goto(cur.saturating_sub(1)),
            Act::Top => st.cur_id = goto(0),
            Act::Bottom => st.cur_id = goto(last),
            Act::HalfDown => st.cur_id = goto((cur + page).min(last)),
            Act::HalfUp => st.cur_id = goto(cur.saturating_sub(page)),
            Act::Next => st.cur_id = goto(step(cur, true)),
            Act::Previous => st.cur_id = goto(step(cur, false)),
            // answer, review: n and p walk the questions
            Act::Note if kind.cards() => st.cur_id = goto(step(cur, true)),
            Act::Prio if kind.cards() => st.cur_id = goto(step(cur, false)),
            Act::Help => st.help = true,
            Act::Open => toggle(&mut st.opened, &st.cur_id),
            Act::Back => st.opened.clear(),
            Act::Focus => st.in_box = !st.in_box,
            Act::Confirm if !st.in_box => st.in_box = true,
            Act::Editor => {
                ratatui::restore();
                let status = Command::new(exe()).arg("--store").arg(root).args(["edit", &st.cur_id]).status();
                ui.term = ratatui::init();
                if let Err(e) = status {
                    st.note = Some((format!("editor: {}", e), true));
                }
            }
            Act::PrevModule => return Ok((decided, Leave::Prev)),
            Act::NextModule => return Ok((decided, Leave::Next)),
            Act::Quit => return Ok((decided, Leave::Quit)),
            Act::QuitHint => st.note = Some((QUIT_HINT.into(), false)),
            Act::Feedback => {
                feedback(ui, &mut screen, cake, store, &format!("{} tree, row {}", kind.name(), st.cur_id))?;
                st.note = screen.note.take();
            }
            Act::Undo => match ui.before.get(&(root.to_path_buf(), st.cur_id.clone())) {
                Some(b) => change = Some(Change::Undo(b.clone())),
                None => st.note = Some(("nothing to undo here".into(), false)),
            },
            Act::Task | Act::Note | Act::Drop | Act::Who | Act::Prio | Act::Subtree | Act::Command | Act::Timer | Act::Complete | Act::Insert | Act::Choose(_) | Act::Confirm
                if !editable =>
            {
                st.note = Some((format!("context: not in {}, no actions here", kind.name()), true));
            }
            // the box: <cr> saves the marked option
            Act::Confirm => {
                if let Some(c) = choose(ui, &mut screen, kind, &item, &all, st.sel)? {
                    change = Some(c);
                    then_next = true;
                }
            }
            Act::Choose(n) if with_options => {
                if let Some(c) = choose(ui, &mut screen, kind, &item, &all, n as usize - 1)? {
                    change = Some(c);
                    then_next = true;
                }
            }
            Act::Insert if with_options => {
                if let Some(c) = choose(ui, &mut screen, kind, &item, &all, n_opts - 1)? {
                    change = Some(c);
                    then_next = true;
                }
            }
            Act::Complete if matches!(kind, Kind::Review | Kind::Mine) => (change, then_next) = (Some(Change::Status("done")), true),
            Act::Drop if kind != Kind::Answer => (change, then_next) = (Some(Change::Status("dropped")), true),
            Act::Timer if kind == Kind::Mine => change = Some(Change::Timer(str_of(&item, "status") != "doing")),
            Act::Who if kind == Kind::News => (change, then_next) = (Some(Change::Hold), true),
            Act::Task if kind == Kind::Sort => change = Some(Change::Status("task")),
            Act::Note if kind == Kind::Sort => (change, then_next) = (Some(Change::Status("note")), true),
            Act::Who if kind.edits() => {
                let at = item["assignee"].as_str().and_then(|w| whos.iter().position(|x| x == w));
                change = Some(Change::Who(Some(whos[at.map_or(0, |a| (a + 1) % whos.len())].clone())));
            }
            Act::Prio if kind.edits() => {
                change = Some(Change::Prio(match item["priority"].as_u64() {
                    None => Some(1),
                    Some(p) if p >= 3 => None,
                    Some(p) => Some(p + 1),
                }))
            }
            Act::Subtree if kind == Kind::Sort => {
                // this row's status and who for every inbox item below it
                let below: Vec<Value> = all
                    .iter()
                    .filter(|i| matches(kind, i, &all, user, since) && str_of(i, "id") != st.cur_id && has_ancestor(&all, i, &st.cur_id))
                    .cloned()
                    .collect();
                let status: &'static str = match str_of(&item, "status") {
                    "task" => "task",
                    "dropped" => "dropped",
                    "" | "note" => "note",
                    _ => "inbox",
                };
                for b in &below {
                    commit(ui, &mut screen, cake, root, b, &Change::Status(status), user, sort_who.as_deref(), &mut decided, &mut st.note)?;
                    if status == "task" {
                        let w = Change::Who(item["assignee"].as_str().map(String::from));
                        commit(ui, &mut screen, cake, root, b, &w, user, sort_who.as_deref(), &mut decided, &mut st.note)?;
                    }
                }
                st.saved = !below.is_empty();
                if st.note.is_none() {
                    st.note = Some((format!("{} items below took this row's status", below.len()), false));
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
                    "due" | "deadline" if kind.edits() => match date(arg) {
                        Ok(v) => change = Some(if cmd == "due" { Change::Due(v) } else { Change::Deadline(v) }),
                        Err(e) => st.note = Some((e, true)),
                    },
                    "who" if kind.edits() => match crate::config::Config::default().who(root, if arg.is_empty() { "-" } else { arg }) {
                        Ok(w) => change = Some(Change::Who(if arg == "me" { Some(user.to_string()) } else { w })),
                        Err(e) => st.note = Some((e.to_string(), true)),
                    },
                    "prio" if kind.edits() => match arg.parse::<u64>() {
                        Ok(p) if (1..=3).contains(&p) => change = Some(Change::Prio(Some(p))),
                        _ if arg.is_empty() || arg == "-" || arg == "0" => change = Some(Change::Prio(None)),
                        _ => st.note = Some(("prio is 1, 2, 3 or - for none".into(), true)),
                    },
                    "move" if kind == Kind::Sort && stores.iter().any(|s| s == arg) => change = Some(Change::Move(arg.to_string())),
                    "move" if kind == Kind::Sort => st.note = Some((format!("no store '{}': {}", arg, stores.join(", ")), true)),
                    other => st.note = Some((format!("unknown command :{}", other), true)),
                }
            }
            _ => (),
        }
        if let Some(c) = change {
            let ok = commit(ui, &mut screen, cake, root, &item, &c, user, sort_who.as_deref(), &mut decided, &mut st.note)?;
            st.saved = ok;
            if ok && then_next {
                st.cur_id = goto(step(cur, true));
                st.in_box = false;
            }
        }
    }
}

/// sorts `all` by the place each item was first seen; new ones go last
/// in the order they come
fn keep_order(seen: &mut std::collections::HashMap<String, usize>, all: &mut [Value]) {
    for i in all.iter() {
        let n = seen.len();
        seen.entry(str_of(i, "id").to_string()).or_insert(n);
    }
    all.sort_by_key(|i| seen[str_of(i, "id")]);
}

fn toggle(set: &mut HashSet<String>, id: &str) {
    if !set.remove(id) {
        set.insert(id.to_string());
    }
}

fn has_ancestor(all: &[Value], i: &Value, id: &str) -> bool {
    let mut p = find(all, str_of(i, "parent"));
    for _ in 0..crate::store::MAX_DEPTH {
        match p {
            Some(x) if str_of(x, "id") == id => return true,
            Some(x) => p = find(all, str_of(x, "parent")),
            None => return false,
        }
    }
    false
}

/// option `pick` of the card as a change; the last one asks for text
fn choose(ui: &mut Ui, screen: &mut Screen, kind: Kind, item: &Value, all: &[Value], pick: usize) -> Result<Option<Change>, Box<dyn Error>> {
    let (_, options) = card_of(kind, item, all);
    if pick >= options.len() {
        return Ok(None);
    }
    if pick == options.len() - 1 {
        let prompt = if kind == Kind::Answer { "your answer:" } else { "what should change:" };
        let Some(text) = ui.input(screen, prompt)?.filter(|t| !t.is_empty()) else { return Ok(None) };
        return Ok(Some(if kind == Kind::Answer { Change::Answer(text) } else { Change::Comment(text) }));
    }
    Ok(Some(match kind {
        Kind::Answer => Change::Answer(answer_text(&body_of(item), &((b'a' + pick as u8) as char).to_string())),
        _ if pick == 0 => Change::Status("done"),
        _ => Change::Status("dropped"),
    }))
}

/// applies one change, remembers the item as changed and how it was;
/// false (and a note) when it failed
#[allow(clippy::too_many_arguments)]
fn commit(
    ui: &mut Ui,
    screen: &mut Screen,
    cake: &Cake,
    root: &Path,
    item: &Value,
    change: &Change,
    user: &str,
    sort_who: Option<&str>,
    decided: &mut usize,
    note: &mut Option<(String, bool)>,
) -> Result<bool, Box<dyn Error>> {
    let id = str_of(item, "id").to_string();
    let key = (root.to_path_buf(), id.clone());
    let result = match ui.save(screen, || apply(cake, item, change, user, sort_who))? {
        Ok(r) => r,
        Err(e) => {
            *note = Some((format!("{}: {}", id, e), true));
            return Ok(false);
        }
    };
    let changed = ui.changed.entry(root.to_path_buf()).or_default();
    if let Change::Undo(_) = change {
        changed.remove(&id);
        ui.before.remove(&key);
        ui.results.remove(&key);
        *decided = decided.saturating_sub(1);
        ui.handled = ui.handled.saturating_sub(1);
    } else {
        if !ui.before.contains_key(&key) {
            changed.insert(id);
            ui.before.insert(key.clone(), before(item));
            *decided += 1;
            ui.handled += 1;
        }
        if !result.is_empty() {
            ui.results.insert(key, result);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn edit_line_shows_values_and_dashes() {
        style::init("unicode");
        let i = json!({"id": "a", "status": "task", "assignee": "jul", "priority": 2, "due": "2026-11-20"});
        assert_eq!(Line::from(edit_line(&i, "jul")).to_string(), "task · who me · prio 2 · due 20.11. · deadline –");
        let i = json!({"id": "a", "status": "inbox"});
        assert_eq!(Line::from(edit_line(&i, "jul")).to_string(), "inbox · who – · prio – · due – · deadline –");
    }

    #[test]
    fn module_items() {
        let all = vec![
            json!({"id": "in", "status": "inbox"}),
            json!({"id": "t0", "status": "task"}),
            json!({"id": "q", "status": "task", "assignee": "jul", "blocks": ["w"]}),
            json!({"id": "w", "status": "task", "assignee": "ai", "blocked_by": ["q"]}),
            json!({"id": "m", "status": "doing", "assignee": "jul"}),
            json!({"id": "r", "status": "review", "owner": "jul"}),
            json!({"id": "n", "status": "task", "assignee": "ai", "by": "claude-code", "created": "2026-11-14T10:00:00Z"}),
        ];
        let ids = |k: Kind| -> Vec<&str> { all.iter().filter(|i| matches(k, i, &all, "jul", 1)).map(|i| str_of(i, "id")).collect() };
        assert_eq!(ids(Kind::Sort), ["in", "t0"]);
        assert_eq!(ids(Kind::Answer), ["q"]);
        assert_eq!(ids(Kind::Mine), ["m"]);
        assert_eq!(ids(Kind::Review), ["r"]);
        assert_eq!(ids(Kind::News), ["n"]);
    }

    #[test]
    fn rows_keep_the_first_seen_order() {
        let mut seen = std::collections::HashMap::new();
        let mut all = vec![json!({"id": "a"}), json!({"id": "b"}), json!({"id": "c"})];
        keep_order(&mut seen, &mut all);
        // after a change, ls sorts b last and a new item d comes first
        let mut all = vec![json!({"id": "d"}), json!({"id": "a"}), json!({"id": "c"}), json!({"id": "b"})];
        keep_order(&mut seen, &mut all);
        let ids: Vec<&str> = all.iter().map(|i| str_of(i, "id")).collect();
        assert_eq!(ids, ["a", "b", "c", "d"]);
    }

    #[test]
    fn card_options_end_with_own_words() {
        style::init("unicode");
        let q = json!({"id": "q", "title": "Raw?", "body": "# Raw?\n\nWhy.\n\n- a) always\n- b) with -d (recommended)"});
        let (ctx, opts) = card_of(Kind::Answer, &q, &[]);
        assert_eq!(ctx, "Why.");
        assert_eq!(opts, ["always", "with -d (recommended)", "… own words"]);
        let (_, opts) = card_of(Kind::Review, &q, &[]);
        assert_eq!(opts, ["ok, done", "drop", "… change something"]);
    }

    #[test]
    #[ignore]
    fn dump_screens_modules() {
        style::init(&std::env::var("CAKE_STYLE").unwrap_or("unicode".into()));
        let item = |id: &str, parent: &str, title: &str, extra: Value| {
            let mut v = json!({"id": id, "title": title, "parent": if parent.is_empty() { Value::Null } else { json!(parent) }, "body": format!("# {}\n\n- why: a line of text\n- done when: it shows", title)});
            for (k, x) in extra.as_object().unwrap() {
                v[k] = x.clone();
            }
            v
        };
        let all = vec![
            item("cli", "", "cake CLI: fast capture", json!({})),
            item("sync", "cli", "Sync errors that say what failed", json!({"status": "task", "assignee": "ai"})),
            item("hint", "sync", "Hint per error type", json!({"status": "task", "assignee": "claude-code", "blocked_by": ["raw"]})),
            item("raw", "hint", "Show git's raw message too?", json!({"status": "task", "assignee": "jul", "blocks": ["hint"], "body": "# Show git's raw message too?\n\nGit's own text helps experts.\n\n- a) always\n- b) only with -d (recommended)"})),
            item("feat", "sync", "Name the failed git step", json!({"status": "review", "owner": "jul"})),
            item("pass", "", "renew passport", json!({"status": "inbox"})),
            item("land", "", "call landlord", json!({"status": "task", "assignee": "jul", "due": "2026-11-20"})),
        ];
        let results = std::collections::HashMap::new();
        for (kind, cur) in [(Kind::Sort, "pass"), (Kind::Answer, "raw"), (Kind::Review, "feat"), (Kind::Mine, "land")] {
            let tree = Tree::new(&all, all.iter().filter(|i| matches(kind, i, &all, "jul", 0)).map(|i| str_of(i, "id").to_string()), 2);
            let rows = tree.rows();
            let at = rows.iter().position(|r| r.id == cur).unwrap();
            let view = View { kind, user: "jul", since: 0, results: &results };
            for width in [60u16, 100] {
                let inner = column_width(width, 100);
                let it = tree.item(cur).unwrap();
                let cols = columns(kind, inner);
                let mut tree = Tree::new(&all, tree.matched.iter().cloned(), 2);
                tree.accent = Some(kind.accent());
                let lines = tree.lines(&rows, at, inner, &|i, c| col_spans(&cols, i, "jul", 0, c), cols_width(&cols), false);
                let (card_title, card, choices) = view.detail(it, &all, true, None, inner);
                let mut tree_lines = vec![Line::from(Span::styled(format!("{} · cake", kind.title()), bold())), cols_header(&cols, inner)];
                tree_lines.extend(lines);
                let screen = Screen { tree: tree_lines, tree_focus: Some(at + 2), card_title, card, choices, keys: keys(&[("<tab>", "details"), ("e", "edit")]), ..Default::default() };
                super::super::screens::dump(kind.name(), width, &screen, 0);
            }
        }
    }
}
