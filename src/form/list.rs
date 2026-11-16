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
    /// the end tab: every open item of the store
    All,
}

impl Kind {
    pub fn of(name: &str) -> Option<Kind> {
        Some(match name {
            "news" => Kind::News,
            "inbox" => Kind::Sort,
            "answer" => Kind::Answer,
            "review" => Kind::Review,
            "mine" => Kind::Mine,
            "all" => Kind::All,
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
            Kind::All => "all",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Kind::News => "what agents did since your last run",
            Kind::Sort => "inbox: new items and tasks without assignee",
            Kind::Answer => "questions for you",
            Kind::Review => "finished work to check",
            Kind::Mine => "your tasks",
            Kind::All => "everything open",
        }
    }

    /// modules where the edit line shows and fields can be set
    fn edits(self) -> bool {
        matches!(self, Kind::Sort | Kind::Mine | Kind::All)
    }

    /// the `:` commands of the module besides w and q: the item's fields
    /// (and move in the inbox); answer has none
    fn commands(self) -> &'static [&'static str] {
        match self {
            Kind::Sort => &["due", "deadline", "who", "prio", "est", "tag", "move", "all"],
            Kind::Mine | Kind::Review | Kind::All => &["due", "deadline", "who", "prio", "est", "tag"],
            // news is read only, answer has its options
            Kind::Answer | Kind::News => &[],
        }
    }

    /// the dim hint of the `:` line: an example per command
    fn command_hint(self) -> String {
        let example = |c: &str| match c {
            "due" => "due fri",
            "deadline" => "deadline 30.11.",
            "who" => "who ai",
            "prio" => "prio 1",
            "est" => "est 2h",
            "tag" => "tag doc,~old",
            "move" => "move <store>",
            "all" => "all",
            _ => "",
        };
        let mut parts: Vec<&str> = self.commands().iter().map(|c| example(c)).collect();
        parts.extend(["filter doc", "w", "q"]);
        parts.join(" · ")
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
            Kind::All => (None, Color::Reset),
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
    tags: Vec<String>,
}

fn tags_of(i: &Value) -> Vec<String> {
    i["tags"].as_array().into_iter().flatten().filter_map(|t| t.as_str()).map(String::from).collect()
}

fn before(i: &Value) -> Before {
    Before {
        status: i["status"].as_str().unwrap_or("note").to_string(),
        who: i["assignee"].as_str().map(String::from),
        prio: i["priority"].as_u64(),
        due: i["due"].as_str().map(String::from),
        deadline: i["deadline"].as_str().map(String::from),
        tags: tags_of(i),
    }
}

/// an item of the ALL tab at filter `level`: 0 open work (inbox, task,
/// doing, review), 1 also notes, 2 also done and dropped
fn in_all(i: &Value, level: u8) -> bool {
    match str_of(i, "status") {
        "inbox" | "task" | "doing" | "review" => true,
        "done" | "dropped" => level >= 2,
        _ => level >= 1,
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
        // read only: what agents did; finished work is in review instead
        Kind::News => status != "review" && news_what(i, user, since).is_some(),
        Kind::Sort => status == "inbox" || (status == "task" && i["assignee"].is_null()),
        // a task for you that someone else's work waits on
        Kind::Answer => status == "task" && str_of(i, "assignee") == user && blocks_open,
        Kind::Review => status == "review" && (str_of(i, "owner") == user || i["owner"].is_null()),
        Kind::Mine => str_of(i, "assignee") == user && matches!(status, "task" | "doing") && !blocks_open,
        Kind::All => !closed(i),
    }
}

/// what the items dir looks like on disk: entries and the newest change;
/// a few stats instead of a cake call, so moving the cursor stays fast
pub(super) fn disk_stamp(root: &Path) -> (usize, std::time::SystemTime) {
    let mut stamp = (0, std::time::UNIX_EPOCH);
    let mut dirs = vec![root.join("items")];
    while let Some(d) = dirs.pop() {
        if let Ok(m) = std::fs::metadata(&d).and_then(|m| m.modified()) {
            stamp.1 = stamp.1.max(m);
        }
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            stamp.0 += 1;
            match e.metadata() {
                Ok(m) if m.is_dir() => dirs.push(e.path()),
                Ok(m) => stamp.1 = stamp.1.max(m.modified().unwrap_or(std::time::UNIX_EPOCH)),
                Err(_) => (),
            }
        }
    }
    stamp
}

/// all items with closed ones only when changed this run; `cake ls` runs
/// again only when the store changed on disk (own saves, agents, sync)
fn load(ui: &mut Ui, cake: &Cake, root: &Path, changed: &HashSet<String>, with_closed: bool) -> Result<Vec<Value>, Box<dyn Error>> {
    let stamp = disk_stamp(root);
    let fresh = match ui.cache.get(root) {
        Some((s, items)) if *s == stamp => items.clone(),
        _ => {
            let items = cake.call(&["ls", "-l", "-A"])?;
            ui.cache.insert(root.to_path_buf(), (stamp, items.clone()));
            items
        }
    };
    Ok(fresh.into_iter().filter(|i| with_closed || !closed(i) || changed.contains(str_of(i, "id"))).collect())
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
            (text, vec!["ok, done".into(), "drop".into(), "talk about it first: back to you, #discuss".into()])
        }
    };
    options.push(format!("{} {}", g.ellipsis, if kind == Kind::Answer { "own words" } else { "change something: the agent reworks it" }));
    (context, options)
}

/// one change to an item, through cake calls
enum Change {
    Status(&'static str),
    Who(Option<String>),
    Prio(Option<u64>),
    Due(Option<String>),
    Deadline(Option<String>),
    /// `a,~b` as for `cake edit --tag`
    Tags(String),
    /// expected work, `-` clears
    Est(String),
    Move(String),
    Timer(bool),
    /// news: back to you; a started item waits as a task
    Hold,
    /// answer: an option's text or own words
    Answer(String),
    /// review: what should change
    Comment(String),
    /// review: back to the user as a task tagged discuss
    Talk,
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
        Change::Talk => {
            // the body keeps the work so far (the review note above); this
            // line says why it is back
            let line = format!("discuss ({}, {})", user, chrono::Local::now().format("%Y-%m-%d"));
            cake.call(&["set", "task", id, "-m", &line])?;
            cake.call(&["edit", id, "--tag", "discuss", "--assignee", user])?;
            "to discuss".into()
        }
        Change::Est(e) => {
            cake.call(&["edit", id, "--est", e])?;
            String::new()
        }
        Change::Tags(t) => {
            cake.call(&["edit", id, "--tag", t])?;
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
            "stopped: back to you".into()
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
            let now = tags_of(item);
            let tags: Vec<String> = b.tags.iter().cloned().chain(now.iter().filter(|t| !b.tags.contains(t)).map(|t| format!("~{}", t))).collect();
            if !tags.is_empty() {
                cake.call(&["edit", id, "--tag", &tags.join(",")])?;
            }
            String::new()
        }
    })
}

/// the set values of an item after its title, ` · ` between them, in the
/// colors of the `:` hint: what (news), status when the mark does not say
/// enough, who, prio, due, deadline; late dates red, near ones yellow;
/// context rows all dim
pub(crate) fn fields(i: &Value, user: &str, since: i64, news: bool, context: bool) -> Vec<Span<'static>> {
    let t = theme::t();
    let today = chrono::Local::now().date_naive();
    let date = |name: &str, d: &str| -> (String, Style) {
        let style = match chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d") {
            Ok(x) if x < today => Style::default().fg(t.error),
            Ok(x) if (x - today).num_days() <= 2 => Style::default().fg(t.doing),
            _ => Style::default(),
        };
        (format!("{}{}", name, short_date(d)), style)
    };
    let mut parts: Vec<(String, Style)> = vec![];
    if news {
        if let Some(w) = news_what(i, user, since) {
            parts.push((w.to_string(), Style::default().fg(t.hint)));
        }
    }
    let status = match (str_of(i, "status"), i["running"].as_bool() == Some(true)) {
        (_, true) => Some(("running", t.doing)),
        ("doing", _) => Some(("doing", t.doing)),
        ("review", _) => Some(("review", t.code)),
        ("done", _) => Some(("done", t.ok)),
        ("dropped", _) => Some(("dropped", t.dim)),
        _ => None,
    };
    if let Some((s, c)) = status {
        parts.push((s.to_string(), Style::default().fg(c)));
    }
    match who_of(i, user) {
        Some(w) if w == "me" => parts.push((w, Style::default())),
        Some(w) => parts.push((w, Style::default().fg(t.hint))),
        None => (),
    }
    match i["priority"].as_u64() {
        Some(1) => parts.push(("p1".into(), Style::default().fg(t.error))),
        Some(2) => parts.push(("p2".into(), Style::default().fg(t.doing))),
        Some(p) => parts.push((format!("p{}", p), Style::default())),
        // from a parent: dim, so own values stand out
        None => {
            if let Some(p) = i["inherited"]["priority"].as_u64() {
                parts.push((format!("p{}", p), dim().add_modifier(Modifier::ITALIC)));
            }
        }
    }
    // expected work of open items: own estimate, ~ from history (dim)
    let hours = |s: i64| if s >= 3600 { format!("{}h{}", s / 3600, if s % 3600 >= 60 { format!("{:02}", s % 3600 / 60) } else { String::new() }) } else { format!("{}m", (s / 60).max(1)) };
    if matches!(str_of(i, "status"), "inbox" | "task" | "doing") {
        if let Some(e) = i["estimate"].as_i64() {
            parts.push((hours(e), Style::default()));
        } else if let Some(e) = i["inherited"]["estimate"].as_i64() {
            parts.push((format!("~{}", hours(e)), dim().add_modifier(Modifier::ITALIC)));
        }
    }
    if let Some(d) = i["due"].as_str() {
        parts.push(date("due ", d));
    }
    if let Some(d) = i["deadline"].as_str() {
        parts.push(date("deadline ", d));
    } else if let Some(d) = i["inherited"]["deadline"].as_str() {
        parts.push((format!("deadline {}", short_date(d)), dim().add_modifier(Modifier::ITALIC)));
    }
    let mut out = vec![];
    for (n, (text, style)) in parts.into_iter().enumerate() {
        if n > 0 {
            out.push(Span::styled(style::g().sep, dim()));
        }
        // muted: never bold; context rows all dim
        out.push(Span::styled(text, if context { dim() } else { style.add_modifier(Modifier::DIM) }));
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
        // the fields are in the tree row, not repeated here; tags may be
        // hidden there (#) or cut
        let mut lines: Vec<Line<'static>> = vec![];
        let tags: Vec<String> = item["tags"].as_array().into_iter().flatten().filter_map(|t| t.as_str()).map(|t| format!("#{}", t)).collect();
        if !tags.is_empty() {
            lines.push(Line::from(Span::styled(tags.join(" "), Style::default().fg(t.tag))));
        }
        let commits: Vec<&str> = item["commits"].as_array().into_iter().flatten().filter_map(|c| c.as_str()).collect();
        if !commits.is_empty() {
            lines.push(Line::from(Span::styled(format!("commits {}", commits.join(" ")), dim())));
        }
        if let Some(what) = news_what(item, self.user, self.since).filter(|_| self.kind == Kind::News) {
            lines.push(Line::from(vec![Span::styled("what ", dim()), Span::raw(what)]));
        }
        if let Some(r) = self.results.get(str_of(item, "id")) {
            lines.push(Line::from(vec![Span::styled("this run ", dim()), Span::styled(r.clone(), Style::default().fg(t.ok))]));
        }
        if !text.trim().is_empty() {
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            lines.extend(markdown::render(&text, width.saturating_sub(4)));
        }
        let choices = options
            .iter()
            .enumerate()
            .flat_map(|(n, text)| {
                let chosen = sel == Some(n);
                let lead = Span::styled(format!("{} {}) ", if chosen { g.select } else { " " }, n + 1), Style::default().fg(t.hint));
                hanging(lead, text, if chosen { bold().fg(t.hint) } else { Style::default() }, width.saturating_sub(4) as usize)
            })
            .collect();
        let title = if actions { str_of(item, "title").to_string() } else { format!("{} · context, no actions here", str_of(item, "title")) };
        (title, lines, choices)
    }
}

/// `lead` then `text` word-wrapped at `width`; wrapped lines start under
/// the text's first word, not under the lead
fn hanging(lead: Span<'static>, text: &str, style: Style, width: usize) -> Vec<Line<'static>> {
    let indent = lead.width();
    let room = width.saturating_sub(indent).max(8);
    let mut rows: Vec<String> = vec![String::new()];
    for word in text.split_whitespace() {
        let row = rows.last_mut().unwrap();
        let len = row.chars().count();
        if len > 0 && len + 1 + word.chars().count() > room {
            rows.push(word.to_string());
        } else {
            if len > 0 {
                row.push(' ');
            }
            row.push_str(word);
        }
    }
    rows.into_iter()
        .enumerate()
        .map(|(n, r)| {
            let head = if n == 0 { lead.clone() } else { Span::raw(" ".repeat(indent)) };
            Line::from(vec![head, Span::styled(r, style)])
        })
        .collect()
}

/// the state of one module run
struct State {
    cur_id: String,
    /// added this run (a, A): shown as open items of the module
    added: HashSet<String>,
    opened: HashSet<String>,
    /// the detail box has the focus
    in_box: bool,
    /// the option marked in the box (answer, review)
    sel: usize,
    /// lines the box text is scrolled
    scroll: u16,
    help: bool,
    /// center the view on the cursor at the next paint (zz)
    center: bool,
    note: Option<(String, bool)>,
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
    ui.module = kind.name();
    let mut st = State { cur_id: String::new(), added: HashSet::new(), opened: HashSet::new(), in_box: false, sel: 0, scroll: 0, help: false, center: false, note: None };
    let mut decided = 0;
    let page = 10usize;
    loop {
        let changed = ui.changed.entry((root.to_path_buf(), kind.name())).or_default().clone();
        let results: std::collections::HashMap<String, String> =
            ui.results.iter().filter(|((r, _), _)| r == root).map(|((_, id), v)| (id.clone(), v.clone())).collect();
        let level = if kind == Kind::All { ui.all_filter } else { 0 };
        let mut all = load(ui, cake, root, &changed, level >= 2)?;
        keep_order(ui.order.entry(root.to_path_buf()).or_default(), &mut all);
        let tagged = |i: &Value| ui.tag_filter.as_ref().map_or(true, |t| i["tags"].as_array().is_some_and(|ts| ts.iter().any(|x| x.as_str() == Some(t))));
        let mut matched: Vec<String> = all
            .iter()
            .filter(|i| tagged(i) && if kind == Kind::All { in_all(i, if ui.tag_filter.is_some() { level.max(1) } else { level }) } else { matches(kind, i, &all, user, since) })
            .map(|i| str_of(i, "id").to_string())
            .collect();
        matched.extend(st.added.iter().filter(|id| !matched.contains(id)).cloned().collect::<Vec<_>>());
        // ALL shows the whole way up, the modules `tree_depth` parents
        let depth = if kind == Kind::All { crate::store::MAX_DEPTH } else { ui.tree_depth };
        let mut tree = Tree::new(&all, matched, depth);
        tree.changed = changed.clone();
        tree.opened = st.opened.clone();
        tree.accent = Some(kind.accent());
        if kind == Kind::All {
            tree.mine = Some(user.to_owned());
        }
        tree.tags = ui.show_tags;
        if let Some(at) = ui.at.take() {
            st.cur_id = at.clone();
            st.opened.insert(at);
            st.center = true;
        }
        let rows = tree.rows();
        if rows.is_empty() && ui.tag_filter.is_some() {
            st.note = Some((format!("nothing with #{} here, filter off", ui.tag_filter.take().unwrap()), true));
            continue;
        }
        if rows.is_empty() {
            return Ok((decided, Leave::Empty));
        }
        // the last open item decided: the module stays, the header marks it
        // done and the note line says how to go on
        // news is read only: nothing to decide, never "done"
        let all_done = !matches!(kind, Kind::All | Kind::News) && !rows.iter().any(|r| r.editable && !r.changed);
        if let Some(d) = ui.done.get_mut(ui.step) {
            *d = all_done;
        }
        if all_done && st.note.is_none() {
            st.note = Some((format!("{} all done here · {} next module", style::g().check, ui.keys.keys_of(Act::NextModule)), false));
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
        // the last column is the scrollbar's
        let lines = tree.lines(&rows, cur, width.saturating_sub(2), &|i, context| fields(i, user, since, kind == Kind::News, context), st.in_box);
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
                Kind::News => vec![("read only:".into(), String::new()), (ks(Act::Timer), "stop agent".into()), (ks(Act::Drop), "drop".into())],
                Kind::All => vec![
                    (ks(Act::Filter), ["more", "closed too", "focus"][ui.all_filter as usize % 3].into()),
                    (format!("{}/{}/{}/{}", ks(Act::Complete), ks(Act::Drop), ks(Act::Note), ks(Act::Task)), "done drop note task".into()),
                    (ks(Act::Who), "who".into()),
                    (ks(Act::Prio), "prio".into()),
                    (":due".into(), String::new()),
                ],
                Kind::Mine => vec![
                    (format!("{}/{}/{}", ks(Act::Timer), ks(Act::Complete), ks(Act::Drop)), "start done drop".into()),
                    (ks(Act::Who), "who".into()),
                    (ks(Act::Prio), "prio".into()),
                ],
                Kind::Answer => vec![("1-9".into(), "pick".into()), (ks(Act::Insert), "own words".into())],
                Kind::Review => vec![
                    (ks(Act::Complete), "ok".into()),
                    (ks(Act::Drop), "drop".into()),
                    (ks(Act::Task), "talk first".into()),
                    (ks(Act::Insert), "rework".into()),
                ],
            }
        };
        if !st.in_box {
            parts.insert(0, (format!("{}/{}", ks(Act::Next), ks(Act::Previous)), "next prev".into()));
            parts.push((ks(Act::Open), "open".into()));
            parts.push((format!("{}/{}", ks(Act::Add), ks(Act::AddChild)), "new".into()));
        }
        parts.extend([
            (ks(Act::Focus), if st.in_box { "tree".into() } else { "details".into() }),
            (ks(Act::Editor), "edit".into()),
            (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module".into()),
        ]);
        parts.push((ks(Act::Quit), "quit".into()));
        // ? keys always last: kept when the line is cut
        parts.push((ks(Act::Help), "keys".into()));
        let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let title = if kind == Kind::All { ["open work, by score", "open work and notes", "everything, closed too"][level as usize % 3] } else { kind.title() };
        let mut head = vec![Span::styled(format!("{} · {}", title, store), bold().fg(kind.accent().1))];
        if let Some(t) = &ui.tag_filter {
            head.push(Span::styled(format!(" · #{}", t), Style::default().fg(theme::t().tag)));
        }
        if kind == Kind::All && !ui.summary.is_empty() {
            head.push(Span::styled(format!(" · {}", ui.summary), dim()));
        }
        let mut tree_lines = vec![Line::from(head)];
        tree_lines.extend(lines);
        let mut screen = Screen {
            tree: tree_lines,
            tree_focus: Some(cur + 1),
            center: std::mem::take(&mut st.center),
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
        st.scroll = st.scroll.min(ui.box_max);
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
                Act::Top if with_options => {
                    st.sel = 0;
                    true
                }
                Act::Bottom if with_options => {
                    st.sel = n_opts - 1;
                    true
                }
                Act::Top => {
                    st.scroll = 0;
                    true
                }
                Act::Bottom => {
                    st.scroll = ui.box_max;
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
            Act::NextModule if kind == Kind::All => st.note = Some(("the last tab: q closes the form".into(), false)),
            // all done: on like the flow goes, past empty modules
            Act::NextModule if all_done || kind == Kind::News => return Ok((decided, Leave::Done)),
            Act::NextModule => return Ok((decided, Leave::Next)),
            Act::Quit => return Ok((decided, Leave::Quit)),
            // the last tab: nothing left to lose, q quits
            Act::Add | Act::AddChild => {
                let parent = (act == Act::AddChild).then(|| st.cur_id.clone());
                if let Some(id) = add_item(ui, &mut screen, cake, store, parent.as_deref())? {
                    // shown as an open item of this module, the cursor on it
                    st.added.insert(id.clone());
                    if let Some(p) = parent {
                        st.opened.insert(p);
                    }
                    st.cur_id = id;
                }
                st.note = screen.note.take();
            }
            Act::Undo => match ui.before.get(&(root.to_path_buf(), st.cur_id.clone())) {
                Some(b) => change = Some(Change::Undo(b.clone())),
                None => st.note = Some(("nothing to undo here".into(), false)),
            },
            Act::Task | Act::Note | Act::Drop | Act::Who | Act::Prio | Act::Timer | Act::Complete | Act::Insert | Act::Choose(_) | Act::Confirm
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
            Act::Complete if matches!(kind, Kind::Review | Kind::Mine | Kind::All) => (change, then_next) = (Some(Change::Status("done")), true),
            Act::Drop if kind != Kind::Answer => (change, then_next) = (Some(Change::Status("dropped")), true),
            Act::Timer if kind == Kind::Mine => change = Some(Change::Timer(str_of(&item, "status") != "doing")),
            Act::Timer if kind == Kind::News => (change, then_next) = (Some(Change::Hold), true),
            Act::Task if matches!(kind, Kind::Sort | Kind::All) => change = Some(Change::Status("task")),
            Act::Task if kind == Kind::Review && editable => (change, then_next) = (Some(Change::Talk), true),
            Act::Note if kind == Kind::Sort => (change, then_next) = (Some(Change::Status("note")), true),
            // ALL: park a task right where it is
            Act::Note if kind == Kind::All => change = Some(Change::Status("note")),
            Act::Filter if kind == Kind::All => ui.all_filter = (ui.all_filter + 1) % 3,
            Act::Center => st.center = true,
            Act::Tags => ui.show_tags = !ui.show_tags,
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
            Act::Command => {
                let Some(line) = ui.input(&mut screen, ":", &kind.command_hint())? else { continue };
                let (cmd, arg) = line.split_once(' ').map_or((line.as_str(), ""), |(c, a)| (c, a.trim()));
                let date = |a: &str| match a {
                    "" | "-" => Ok(None),
                    a => crate::util::parse_date(a).map(Some),
                };
                let ours = kind.commands().contains(&cmd);
                match cmd {
                    "" => (),
                    "q" | "q!" | "wq" | "x" => return Ok((decided, Leave::Quit)),
                    "w" => return Ok((decided, Leave::Done)),
                    // a filter, not a change: works on any row
                    "filter" => ui.tag_filter = Some(arg.trim_start_matches('#').to_string()).filter(|t| !t.is_empty()),
                    _ if ours && !editable => st.note = Some((format!("context: not in {}, no actions here", kind.name()), true)),
                    "due" | "deadline" if ours => match date(arg) {
                        Ok(v) => change = Some(if cmd == "due" { Change::Due(v) } else { Change::Deadline(v) }),
                        Err(e) => st.note = Some((e, true)),
                    },
                    "who" if ours => match crate::config::Config::default().who(root, if arg.is_empty() { "-" } else { arg }) {
                        Ok(w) => change = Some(Change::Who(if arg == "me" { Some(user.to_string()) } else { w })),
                        Err(e) => st.note = Some((e.to_string(), true)),
                    },
                    "prio" if ours => match arg.parse::<u64>() {
                        Ok(p) if (1..=3).contains(&p) => change = Some(Change::Prio(Some(p))),
                        _ if arg.is_empty() || arg == "-" || arg == "0" => change = Some(Change::Prio(None)),
                        _ => st.note = Some(("prio is 1, 2, 3 or - for none".into(), true)),
                    },
                    "est" if ours => change = Some(Change::Est(if arg.is_empty() { "-".into() } else { arg.to_string() })),
                    "tag" if ours && !arg.is_empty() => change = Some(Change::Tags(arg.split([',', ' ']).filter(|t| !t.is_empty()).map(|t| t.trim_start_matches('#')).collect::<Vec<_>>().join(","))),
                    "tag" if ours => st.note = Some(("tag doc adds, tag ~doc removes; filter doc shows only doc".into(), true)),
                    "move" if ours && stores.iter().any(|s| s == arg) => change = Some(Change::Move(arg.to_string())),
                    "move" if ours => st.note = Some((format!("no store '{}': {}", arg, stores.join(", ")), true)),
                    "all" if ours => {
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
                        if st.note.is_none() {
                            st.note = Some((format!("{} items below took this row's status", below.len()), false));
                        }
                    }
                    other => st.note = Some((format!("unknown command :{} in {}; here: {}", other, kind.name(), kind.command_hint()), true)),
                }
            }
            _ => (),
        }
        if let Some(c) = change {
            let ok = commit(ui, &mut screen, cake, root, &item, &c, user, sort_who.as_deref(), &mut decided, &mut st.note)?;
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
        let Some(text) = ui.input(screen, prompt, "")?.filter(|t| !t.is_empty()) else { return Ok(None) };
        return Ok(Some(if kind == Kind::Answer { Change::Answer(text) } else { Change::Comment(text) }));
    }
    Ok(Some(match kind {
        Kind::Answer => Change::Answer(answer_text(&body_of(item), &((b'a' + pick as u8) as char).to_string())),
        _ if pick == 0 => Change::Status("done"),
        _ if pick == 1 => Change::Status("dropped"),
        _ => Change::Talk,
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
    let changed = ui.changed.entry((root.to_path_buf(), ui.module)).or_default();
    if let Change::Undo(_) = change {
        changed.remove(&id);
        ui.before.remove(&key);
        ui.results.remove(&key);
        *decided = decided.saturating_sub(1);
        if ui.module != "news" {
            ui.handled = ui.handled.saturating_sub(1);
        }
    } else {
        if !ui.before.contains_key(&key) {
            changed.insert(id);
            ui.before.insert(key.clone(), before(item));
            *decided += 1;
            // news is read only: not in the progress
            if ui.module != "news" {
                ui.handled += 1;
            }
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
        let finished = vec![json!({"id": "f", "status": "review", "by": "claude-code", "created": "2026-11-14T10:00:00Z"})];
        assert!(!matches(Kind::News, &finished[0], &finished, "jul", 1), "finished work is in review, not news");
    }

    #[test]
    fn all_filter_levels() {
        let ids = |level: u8| -> Vec<&str> {
            ["inbox", "task", "doing", "review", "", "done", "dropped"].into_iter().filter(|s| in_all(&json!({"status": if s.is_empty() { Value::Null } else { json!(s) }}), level)).collect()
        };
        assert_eq!(ids(0), ["inbox", "task", "doing", "review"]);
        assert_eq!(ids(1), ["inbox", "task", "doing", "review", ""]);
        assert_eq!(ids(2).len(), 7);
    }

    #[test]
    fn command_hints_list_only_what_works() {
        assert_eq!(Kind::Review.command_hint(), "due fri · deadline 30.11. · who ai · prio 1 · est 2h · tag doc,~old · filter doc · w · q");
        assert_eq!(Kind::Sort.command_hint(), "due fri · deadline 30.11. · who ai · prio 1 · est 2h · tag doc,~old · move <store> · all · filter doc · w · q");
        assert_eq!(Kind::Answer.command_hint(), "filter doc · w · q");
    }

    #[test]
    fn wrapped_options_hang_under_their_first_word() {
        let lines = hanging(Span::raw("› 1) "), "one two three four five", Style::default(), 18);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        assert_eq!(text, ["› 1) one two three", "     four five"]);
    }

    #[test]
    fn tree_fields_show_only_set_values() {
        style::init("unicode");
        let text = |i: &Value| Line::from(fields(i, "jul", 0, false, false)).to_string();
        let i = json!({"id": "a", "status": "task", "assignee": "jul", "priority": 1, "due": "2026-11-20"});
        assert_eq!(text(&i), "me · p1 · due 20.11.");
        assert_eq!(text(&json!({"id": "a", "status": "inbox"})), "");
        assert_eq!(text(&json!({"id": "a", "status": "review", "assignee": "claude-code"})), "review · claude-code");
        let inh = json!({"id": "a", "status": "task", "inherited": {"priority": 1, "deadline": "2026-12-01"}});
        assert_eq!(text(&inh), "p1 · deadline 01.12.");
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
        assert_eq!(opts, ["ok, done", "drop", "talk about it first: back to you, #discuss", "… change something: the agent reworks it"]);
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
                let mut tree = Tree::new(&all, tree.matched.iter().cloned(), 2);
                tree.accent = Some(kind.accent());
                let lines = tree.lines(&rows, at, inner, &|i, c| fields(i, "jul", 0, kind == Kind::News, c), false);
                let (card_title, card, choices) = view.detail(it, &all, true, None, inner);
                let mut tree_lines = vec![Line::from(Span::styled(format!("{} · cake", kind.title()), bold()))];
                tree_lines.extend(lines);
                let screen = Screen { tree: tree_lines, tree_focus: Some(at + 1), card_title, card, choices, keys: keys(&[("<tab>", "details"), ("e", "edit")]), ..Default::default() };
                super::super::screens::dump(kind.name(), width, &screen, 0);
            }
        }
    }
}
