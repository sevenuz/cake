//! table modules: one row per item, one key decides and moves on, enter
//! saves all decisions. news (what agents did), sort (inbox and tasks
//! without assignee), mine (your open tasks).

use super::*;
use crate::util;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    News,
    Sort,
    Mine,
}

/// a decision for one row; what a key set
#[derive(Clone, Default)]
struct Row {
    id: String,
    title: String,
    prefix: String,
    status: String,
    /// news: new, started, finished
    what: String,
    /// sort: note, task, drop; news: ok, hold, drop; mine: start, stop, done, drop, later
    action: Option<&'static str>,
    who: Option<String>,
    /// who it was before, to see a change
    orig_who: Option<String>,
    due: Option<String>,
    deadline: Option<String>,
    prio: Option<u64>,
    move_to: Option<String>,
    body: String,
}

fn ts(v: &Value) -> i64 {
    v.as_str().and_then(|t| util::parse_timestamp(t).ok()).unwrap_or(0)
}

/// rows of `kind` in tree order, with tree prefixes for context
fn rows(cake: &Cake, kind: Kind, user: &str, since: i64) -> Result<Vec<Row>, Box<dyn Error>> {
    let all = cake.call(&["ls", "-l"])?;
    let blocks_open = |i: &Value| {
        i["blocks"].as_array().is_some_and(|b| {
            b.iter().any(|b| find(&all, b.as_str().unwrap_or("")).is_some_and(|x| str_of(x, "status") != "done"))
        })
    };
    let mut out = vec![];
    for i in &all {
        let status = str_of(i, "status").to_string();
        let mine = str_of(i, "assignee") == user;
        let (keep, what) = match kind {
            Kind::Sort => (status == "inbox" || (status == "task" && i["assignee"].is_null()), String::new()),
            Kind::Mine => (mine && matches!(status.as_str(), "task" | "doing") && !blocks_open(i), String::new()),
            Kind::News => {
                let agent = str_of(i, "by") != user && !i["by"].is_null();
                let started = i["time"].as_array().is_some_and(|t| t.iter().any(|t| ts(&t["start"]) >= since));
                let what = if ts(&i["created"]) >= since {
                    "new"
                } else if status == "review" && started {
                    "finished"
                } else if started {
                    "started"
                } else {
                    ""
                };
                (agent && !what.is_empty() && since > 0, what.to_string())
            }
        };
        if !keep {
            continue;
        }
        let depth = {
            let mut d = 0;
            let mut cur = str_of(i, "parent").to_string();
            while let Some(p) = find(&all, &cur) {
                d += 1;
                cur = str_of(p, "parent").to_string();
                if d > 8 {
                    break;
                }
            }
            d
        };
        let parent = find(&all, str_of(i, "parent")).map(|p| str_of(p, "title").to_string());
        out.push(Row {
            id: str_of(i, "id").to_string(),
            title: str_of(i, "title").to_string(),
            // shown as a group heading above the row, not in its title
            prefix: match (depth, parent) {
                (0, _) | (_, None) => String::new(),
                (_, Some(p)) => p,
            },
            status,
            what,
            action: None,
            who: i["assignee"].as_str().map(String::from),
            orig_who: i["assignee"].as_str().map(String::from),
            due: i["due"].as_str().map(String::from),
            deadline: i["deadline"].as_str().map(String::from),
            prio: i["priority"].as_u64(),
            move_to: None,
            body: body_of(i),
        });
    }
    Ok(out)
}

pub(super) fn count(cake: &Cake, kind: Kind, user: &str, since: i64) -> Result<usize, Box<dyn Error>> {
    Ok(rows(cake, kind, user, since)?.len())
}

fn cut(s: &str, w: usize) -> String {
    if s.chars().count() <= w {
        return format!("{:<w$}", s, w = w);
    }
    let g = style::g();
    let keep = w.saturating_sub(g.ellipsis.chars().count());
    format!("{}{}", s.chars().take(keep).collect::<String>(), g.ellipsis)
}

/// short date for a column: 30.11.
fn short_date(d: &Option<String>) -> String {
    d.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..d.len().min(10)], "%Y-%m-%d").ok())
        .map_or(String::new(), |d| d.format("%d.%m.").to_string())
}

/// the table's lines, and which line holds the cursor
fn render(kind: Kind, rows: &[Row], cur: usize, opened: Option<usize>, width: u16, user: &str) -> (Vec<Line<'static>>, usize) {
    let t = theme::t();
    let g = style::g();
    let w = width as usize;
    let mut out = vec![];
    let (head, fixed) = match kind {
        Kind::Sort => (format!("  {:<6} {{}} {:<9} {:<7} {:<8} !", "kind", "who", "due", "deadline"), 2 + 7 + 10 + 8 + 9 + 2),
        Kind::News => (format!("  {:<6} {:<9} {{}} {:<6}", "do", "what", "status"), 2 + 7 + 10 + 7),
        Kind::Mine => (format!("  {:<6} {{}} {:<9} {:<7} {:<8} !", "do", "who", "due", "deadline"), 2 + 7 + 10 + 8 + 9 + 2),
    };
    let title_w = w.saturating_sub(fixed).max(10);
    out.push(Line::from(Span::styled(head.replace("{}", &cut("title", title_w)), dim())));
    let mut group = String::new();
    let mut focus = 0;
    for (n, r) in rows.iter().enumerate() {
        if r.prefix != group {
            group = r.prefix.clone();
            if !group.is_empty() {
                out.push(Line::from(Span::styled(format!("  {}", cut(&group, w.saturating_sub(2)).trim_end()), bold().fg(t.dim))));
            }
        }
        let indent = if group.is_empty() { "" } else { "  " };
        let here = n == cur;
        if here {
            focus = out.len();
        }
        let mark = if here { g.select } else { " " };
        let action = r.action.map_or(g.note.to_string(), |a| a.to_string());
        let who = |r: &Row| r.who.as_deref().map(|w| if w == user { "me" } else { w }).unwrap_or("").to_string();
        let title = cut(&format!("{}{}", indent, r.title), title_w);
        let text = match kind {
            Kind::Sort => format!(
                "{} {:<6} {} {:<9} {:<7} {:<8} {}",
                mark,
                if let Some(m) = &r.move_to { format!("→{}", m) } else { action },
                title,
                cut(&who(r), 9),
                short_date(&r.due),
                short_date(&r.deadline),
                r.prio.map_or("-".to_string(), |p| p.to_string())
            ),
            Kind::News => format!("{} {:<6} {:<9} {} {:<6}", mark, action, r.what, title, r.status),
            Kind::Mine => format!(
                "{} {:<6} {} {:<9} {:<7} {:<8} {}",
                mark,
                action,
                title,
                cut(&who(r), 9),
                short_date(&r.due),
                short_date(&r.deadline),
                r.prio.map_or("-".to_string(), |p| p.to_string())
            ),
        };
        let style = if here {
            bold().fg(t.hint)
        } else if r.action.is_some() || r.move_to.is_some() {
            Style::default()
        } else {
            Style::default().fg(t.dim)
        };
        out.push(Line::from(Span::styled(text, style)));
        if opened == Some(n) {
            let body = if r.body.is_empty() { "(no text)".to_string() } else { r.body.clone() };
            for l in markdown::render(&body, width.saturating_sub(6)) {
                let mut spans = vec![Span::raw("      ")];
                spans.extend(l.spans.into_iter().map(|s| Span::styled(s.content.to_string(), s.style.patch(dim()))));
                out.push(Line::from(spans));
            }
        }
    }
    (out, focus)
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

/// runs one table module in one store; returns (decided, quit)
pub(super) fn run(
    ui: &mut Ui,
    cake: &Cake,
    root: &Path,
    kind: Kind,
    user: &str,
    store: &str,
    since: i64,
) -> Result<(usize, Leave), Box<dyn Error>> {
    let mut rows = rows(cake, kind, user, since)?;
    if rows.is_empty() {
        return Ok((0, Leave::Empty));
    }
    if kind == Kind::Sort {
        // tasks without assignee are sorted already, they need a who
        for r in rows.iter_mut().filter(|r| r.status == "task") {
            r.action = Some("task");
        }
    }
    let stores: Vec<String> = crate::stores::all()?.into_iter().map(|(n, _)| n).filter(|n| n != store).collect();
    let whos = who_cycle(root, user);
    let (mut cur, mut opened, mut help, mut note) = (0usize, None::<usize>, false, None::<(String, bool)>);
    let mut quit_armed = false;
    let page = 10usize;
    loop {
        let width = column_width(ui.term.size()?.width, ui.max_width).saturating_sub(4);
        let k = &ui.keys;
        let ks = |a: Act| k.keys_of(a);
        let parts: Vec<(String, String)> = match kind {
            Kind::Sort => vec![
                (format!("{}/{}/{}", ks(Act::Task), ks(Act::Note), ks(Act::Drop)), "task note drop".into()),
                (ks(Act::Who), "who".into()),
                (ks(Act::Prio), "prio".into()),
                (":due :deadline :move".into(), "".into()),
                (ks(Act::Open), "open".into()),
                (ks(Act::Confirm), "save".into()),
                (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module".into()),
                (ks(Act::Help), "keys".into()),
            ],
            Kind::News => vec![
                (ks(Act::Who), "who (hold: back to you)".into()),
                (ks(Act::Drop), "drop".into()),
                (ks(Act::Open), "open".into()),
                (ks(Act::Confirm), "save".into()),
                (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module".into()),
            ],
            Kind::Mine => vec![
                (format!("{}/{}/{}", ks(Act::Timer), ks(Act::Complete), ks(Act::Drop)), "start done drop".into()),
                (ks(Act::Who), "who".into()),
                (ks(Act::Open), "open".into()),
                (ks(Act::Confirm), "save".into()),
                (format!("{}/{}", ks(Act::PrevModule), ks(Act::NextModule)), "module".into()),
            ],
        };
        let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let (lines, focus) = render(kind, &rows, cur, opened, width, user);
        let mut screen = Screen {
            card_title: format!(
                "{} · {}",
                match kind {
                    Kind::Sort => "inbox and tasks without assignee",
                    Kind::News => "what agents did since your last run",
                    Kind::Mine => "your tasks",
                },
                store
            ),
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
        let decided = rows.iter().filter(|r| r.action.is_some() || r.move_to.is_some()).count();
        let last = rows.len() - 1;
        let mut next = false;
        match act {
            Act::Down => cur = (cur + 1).min(last),
            Act::Up => cur = cur.saturating_sub(1),
            Act::Top => cur = 0,
            Act::Bottom => cur = last,
            Act::HalfDown => cur = (cur + page).min(last),
            Act::HalfUp => cur = cur.saturating_sub(page),
            Act::Help => help = true,
            Act::Open => opened = if opened == Some(cur) { None } else { Some(cur) },
            Act::Back => opened = None,
            // switching module saves, like leaving a field
            Act::PrevModule | Act::NextModule => {
                let n = save(ui, &mut screen, cake, kind, &rows, user)?;
                return Ok((n, if act == Act::PrevModule { Leave::Prev } else { Leave::Next }));
            }
            Act::Quit => {
                if decided > 0 && !quit_armed {
                    quit_armed = true;
                    note = Some(("unsaved decisions: <cr> saves, q again quits without them".into(), true));
                    continue;
                }
                return Ok((0, Leave::Quit));
            }
            Act::Feedback => {
                let place = format!("{} table, row {}", if kind == Kind::Sort { "sort" } else if kind == Kind::News { "news" } else { "mine" }, rows[cur].id);
                feedback(ui, &mut screen, cake, store, &place)?;
                note = screen.note.take();
            }
            Act::Note if kind == Kind::Sort => (rows[cur].action, next) = (Some("note"), true),
            // task stays on the row: who and dates usually follow
            Act::Task if kind == Kind::Sort => {
                rows[cur].action = Some("task");
                rows[cur].who.get_or_insert(user.to_string());
            }
            Act::Drop => (rows[cur].action, next) = (Some("drop"), true),
            Act::Complete if kind == Kind::Mine => (rows[cur].action, next) = (Some("done"), true),
            Act::Timer if kind == Kind::Mine => {
                let start = rows[cur].status != "doing";
                (rows[cur].action, next) = (Some(if start { "start" } else { "stop" }), true)
            }
            Act::Who => {
                let row = &mut rows[cur];
                let at = row.who.as_ref().and_then(|w| whos.iter().position(|x| x == w));
                row.who = Some(whos[at.map_or(0, |a| (a + 1) % whos.len())].clone());
                if row.action.is_none() {
                    row.action = Some(match kind {
                        Kind::Sort => "task",
                        _ => "assign",
                    });
                }
            }
            Act::Prio if kind != Kind::News => {
                let row = &mut rows[cur];
                row.prio = match row.prio {
                    None => Some(1),
                    Some(3) => None,
                    Some(p) => Some(p + 1),
                };
                if row.action.is_none() {
                    row.action = Some(if kind == Kind::Sort { "task" } else { "assign" });
                    row.who.get_or_insert(user.to_string());
                }
            }
            Act::Undo => {
                rows[cur].action = None;
                rows[cur].move_to = None;
            }
            Act::Subtree if kind == Kind::Sort => {
                let r = rows[cur].clone();
                let below: Vec<usize> = rows.iter().enumerate().filter(|(_, x)| x.prefix == r.title && x.id != r.id).map(|(n, _)| n).collect();
                for n in &below {
                    let x = &mut rows[*n];
                    x.action = r.action;
                    x.who = r.who.clone();
                    x.move_to = r.move_to.clone();
                }
                note = Some((format!("{} rows below took this decision", below.len()), false));
            }
            Act::Command => {
                let Some(line) = ui.input(&mut screen, ":")? else { continue };
                let (cmd, arg) = line.split_once(' ').map_or((line.as_str(), ""), |(c, a)| (c, a.trim()));
                match cmd {
                    "w" => {
                        let n = save(ui, &mut screen, cake, kind, &rows, user)?;
                        return Ok((n, Leave::Done));
                    }
                    "wq" | "x" => {
                        let n = save(ui, &mut screen, cake, kind, &rows, user)?;
                        return Ok((n, Leave::Quit));
                    }
                    "q" if decided > 0 => note = Some(("unsaved decisions: :w saves, :q! quits without them".into(), true)),
                    "q" | "q!" => return Ok((0, Leave::Quit)),
                    "due" | "deadline" if kind != Kind::News => {
                        let value = match arg {
                            "" | "-" => Ok(None),
                            a => util::parse_date(a).map(Some),
                        };
                        match value {
                            Ok(v) => {
                                let row = &mut rows[cur];
                                if cmd == "due" { row.due = v } else { row.deadline = v }
                                if row.action.is_none() {
                                    row.action = Some(if kind == Kind::Sort { "task" } else { "assign" });
                                    row.who.get_or_insert(user.to_string());
                                }
                            }
                            Err(e) => note = Some((e, true)),
                        }
                    }
                    "who" => match crate::config::Config::default().who(root, arg) {
                        Ok(w) => {
                            let w = if arg == "me" { Some(user.to_string()) } else { w };
                            rows[cur].who = w;
                            if rows[cur].action.is_none() {
                                rows[cur].action = Some(if kind == Kind::Sort { "task" } else { "assign" });
                            }
                        }
                        Err(e) => note = Some((e.to_string(), true)),
                    },
                    "prio" => match arg.parse::<u64>() {
                        Ok(p) if (1..=3).contains(&p) => rows[cur].prio = Some(p),
                        Ok(0) | Err(_) if arg.is_empty() || arg == "-" || arg == "0" => rows[cur].prio = None,
                        _ => note = Some(("prio is 1, 2, 3 or - for none".into(), true)),
                    },
                    "move" if kind == Kind::Sort => {
                        if stores.iter().any(|s| s == arg) {
                            rows[cur].move_to = Some(arg.to_string());
                        } else {
                            note = Some((format!("no store '{}': {}", arg, stores.join(", ")), true));
                        }
                    }
                    other => note = Some((format!("unknown command :{}", other), true)),
                }
            }
            Act::Confirm => {
                let n = save(ui, &mut screen, cake, kind, &rows, user)?;
                return Ok((n, Leave::Done));
            }
            _ => (),
        }
        if next {
            cur = (cur + 1).min(last);
        }
    }
}

/// applies every decided row through cake calls; returns how many
fn save(ui: &mut Ui, screen: &mut Screen, cake: &Cake, kind: Kind, rows: &[Row], user: &str) -> Result<usize, Box<dyn Error>> {
    let mut done = 0;
    let mut errors = vec![];
    for r in rows.iter().filter(|r| r.action.is_some() || r.move_to.is_some()) {
        let id = r.id.as_str();
        let result = ui.save(screen, || -> Result<(), Box<dyn Error>> {
            match (kind, r.action) {
                (Kind::Sort, Some("note")) => {
                    cake.call(&["note", id])?;
                }
                (Kind::Sort, Some("drop")) | (Kind::News, Some("drop")) | (Kind::Mine, Some("drop")) => {
                    cake.call(&["drop", id])?;
                }
                (Kind::Sort, Some("task")) => {
                    cake.call(&["do", id])?;
                    let who = r.who.clone().unwrap_or_else(|| user.to_string());
                    let mut args = vec!["edit", id, "--assignee", who.as_str()];
                    let (due, deadline, prio) = (
                        r.due.clone().unwrap_or("-".into()),
                        r.deadline.clone().unwrap_or("-".into()),
                        r.prio.unwrap_or(0).to_string(),
                    );
                    args.extend(["--due", &due, "--deadline", &deadline, "--prio", &prio]);
                    cake.call(&args)?;
                }
                (Kind::Mine, Some("start")) => {
                    cake.call(&["start", id])?;
                }
                (Kind::Mine, Some("stop")) => {
                    cake.call(&["stop", id])?;
                }
                (Kind::Mine, Some("done")) => {
                    cake.call(&["done", id])?;
                }
                (Kind::Mine, Some("assign")) => (),
                _ => (),
            }
            // a new who (news, mine; sort sets it with the task)
            if kind != Kind::Sort && r.who != r.orig_who {
                if let Some(w) = &r.who {
                    cake.call(&["assign", id, w])?;
                    // held back: a started item waits as a task
                    if kind == Kind::News && r.status == "doing" {
                        cake.call(&["do", id])?;
                    }
                }
            }
            if let Some(to) = &r.move_to {
                cake.call(&["mv", id, "--to-store", to])?;
            }
            Ok(())
        })?;
        match result {
            Ok(()) => done += 1,
            Err(e) => errors.push(format!("{}: {}", id, e)),
        }
    }
    ui.handled += done;
    if !errors.is_empty() {
        screen.note = Some((errors.join("; "), true));
        ui.draw(screen)?;
        key()?;
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_table_renders_in_80_columns() {
        let row = |id: &str, title: &str, prefix: &str| Row { id: id.into(), title: title.into(), prefix: prefix.into(), ..Default::default() };
        let mut rows = vec![
            row("pass", "renew passport", ""),
            row("blog", "blog post about cake", ""),
            row("sync", "sync errors unclear", ""),
            row("step", "name the failed git step", "sync errors unclear"),
            row("land", "call landlord", ""),
        ];
        rows[0].action = Some("task");
        rows[0].who = Some("jul".into());
        rows[0].due = Some("2026-11-20".into());
        rows[1].action = Some("note");
        rows[2].action = Some("task");
        rows[2].who = Some("ai".into());
        rows[2].deadline = Some("2026-11-30".into());
        rows[2].prio = Some(2);
        rows[3].move_to = Some("paper".into());
        let screen = Screen {
            card_title: "inbox and tasks without assignee · cake".into(),
            card: render(Kind::Sort, &rows, 4, Some(2), 74, "jul").0,
            keys: keys(&[("n/t/x", "note task drop"), ("w", "who"), ("enter", "save")]),
            ..Default::default()
        };
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 22)).unwrap();
        let names: Vec<String> = ["news", "sort", "answer"].iter().map(|s| s.to_string()).collect();
        term.draw(|f| paint(f, &header(&names, &[0, 5, 1], 1, 1, 6, 72), &screen, 1, 100)).unwrap();
        let buf = term.backend().buffer().clone();
        let lines: Vec<String> = (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
            .collect();
        println!("{}", lines.join("\n"));
        assert!(lines.iter().any(|l| l.contains("task") && l.contains("renew passport") && l.contains("me") && l.contains("20.11.")));
        assert!(lines.iter().any(|l| l.contains("→paper")));
    }

    #[test]
    #[ignore]
    fn dump_screens_tables() {
        style::init(&std::env::var("CAKE_STYLE").unwrap_or("unicode".into()));
        let keymap = Keymap::new(&Default::default()).unwrap();
        let k = |a: Act| keymap.keys_of(a);
        let row = |id: &str, title: &str, prefix: &str| Row { id: id.into(), title: title.into(), prefix: prefix.into(), ..Default::default() };
        let mut sort = vec![
            row("pass", "renew passport", ""),
            row("blog", "blog post about cake", ""),
            row("dash", "Dashboard: one screen for what runs", ""),
            row("sect", "dashboard should come after the form", "Dashboard: one screen for what runs"),
            row("land", "call landlord", ""),
        ];
        sort[0].action = Some("task");
        sort[0].who = Some("jul".into());
        sort[0].due = Some("2026-11-20".into());
        sort[1].action = Some("note");
        sort[2].action = Some("task");
        sort[2].who = Some("ai".into());
        sort[2].deadline = Some("2026-11-30".into());
        sort[2].prio = Some(2);
        let mut news = vec![row("hint", "Hint per error type", "Sync errors"), row("step", "Name the failed git step", "Sync errors")];
        news[0].what = "new".into();
        news[0].status = "task".into();
        news[1].what = "finished".into();
        news[1].status = "review".into();
        news[1].who = Some("jul".into());
        news[1].action = Some("assign");
        let mut mine = vec![row("land", "call landlord", ""), row("pass", "renew passport", "")];
        mine[0].who = Some("jul".into());
        mine[0].action = Some("start");
        mine[1].who = Some("ai".into());
        mine[1].action = Some("assign");
        mine[1].due = Some("2026-11-20".into());
        for width in [60u16, 100] {
            let inner = column_width(width, 100).saturating_sub(4);
            let screen = Screen {
                card_title: "inbox and tasks without assignee · cake".into(),
                card: render(Kind::Sort, &sort, 4, None, inner, "jul").0,
                keys: keys(&[(&format!("{}/{}/{}", k(Act::Task), k(Act::Note), k(Act::Drop)), "task note drop"), (&k(Act::Who), "who"), (&k(Act::Prio), "prio"), (":due :deadline :move", ""), (&k(Act::Open), "open"), (&k(Act::Confirm), "save"), (&k(Act::Help), "keys")]),
                ..Default::default()
            };
            super::super::screens::dump("sort", width, &screen, 1);
            let screen = Screen {
                card_title: "what agents did since your last run · cake".into(),
                card: render(Kind::News, &news, 0, Some(0), inner, "jul").0,
                keys: keys(&[(&k(Act::Who), "who (hold: back to you)"), (&k(Act::Drop), "drop"), (&k(Act::Open), "open"), (&k(Act::Confirm), "save"), (&k(Act::Back), "skip")]),
                ..Default::default()
            };
            super::super::screens::dump("news", width, &screen, 0);
            let screen = Screen {
                card_title: "your tasks · cake".into(),
                card: render(Kind::Mine, &mine, 1, None, inner, "jul").0,
                keys: keys(&[(&format!("{}/{}/{}", k(Act::Timer), k(Act::Complete), k(Act::Drop)), "start done drop"), (&k(Act::Who), "who"), (&k(Act::Open), "open"), (&k(Act::Confirm), "save"), (&k(Act::Back), "skip")]),
                ..Default::default()
            };
            super::super::screens::dump("mine", width, &screen, 4);
        }
    }
}
