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
            prefix: match (depth, parent) {
                (0, _) | (_, None) => String::new(),
                (_, Some(p)) => format!("{} › ", p),
            },
            status,
            what,
            action: None,
            who: i["assignee"].as_str().map(String::from),
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

fn render(kind: Kind, rows: &[Row], cur: usize, opened: Option<usize>, width: u16, user: &str) -> Vec<Line<'static>> {
    let t = theme::t();
    let g = style::g();
    let w = width as usize;
    let mut out = vec![];
    let (head, fixed) = match kind {
        Kind::Sort => (format!("  {:<6} {{}} {:<9} {:<7} {:<8} !", "kind", "who", "due", "deadline"), 2 + 7 + 10 + 8 + 9 + 2),
        Kind::News => (format!("  {:<6} {:<9} {{}} {:<6}", "do", "what", "status"), 2 + 7 + 10 + 7),
        Kind::Mine => (format!("  {:<6} {{}} {:<7} {:<8} !", "do", "due", "deadline"), 2 + 7 + 8 + 9 + 2),
    };
    let title_w = w.saturating_sub(fixed).max(10);
    out.push(Line::from(Span::styled(head.replace("{}", &cut("title", title_w)), dim())));
    for (n, r) in rows.iter().enumerate() {
        let here = n == cur;
        let mark = if here { g.select } else { " " };
        let action = r.action.map_or(g.note.to_string(), |a| a.to_string());
        let who = |r: &Row| r.who.as_deref().map(|w| if w == user { "me" } else { w }).unwrap_or("").to_string();
        let title = cut(&format!("{}{}", r.prefix, r.title), title_w);
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
                "{} {:<6} {} {:<7} {:<8} {}",
                mark,
                action,
                title,
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
    out
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
) -> Result<(usize, bool), Box<dyn Error>> {
    let mut rows = rows(cake, kind, user, since)?;
    if rows.is_empty() {
        return Ok((0, false));
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
    loop {
        let width = column_width(ui.term.size()?.width, ui.max_width).saturating_sub(4);
        let key_line = match kind {
            Kind::Sort => keys(&[
                ("n/t/x", "note task drop"),
                ("w", "who"),
                ("d/D", "due deadline"),
                ("p", "prio"),
                ("m", "move"),
                ("A", "subtree"),
                ("o", "open"),
                ("enter", "save"),
                ("?", "keys"),
            ]),
            Kind::News => keys(&[("o/h/x", "ok hold drop"), ("v", "view"), ("enter", "save"), ("esc", "skip"), ("?", "keys")]),
            Kind::Mine => keys(&[("s/d/x/l", "start done drop later"), ("o", "open"), ("enter", "save"), ("esc", "skip")]),
        };
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
            card: render(kind, &rows, cur, opened, width, user),
            keys: key_line,
            note: note.take(),
            help,
            ..Default::default()
        };
        ui.draw(&screen)?;
        let k = key()?;
        if ctrl_c(&k) {
            return Ok((0, true));
        }
        if help {
            help = false;
            continue;
        }
        let decided = rows.iter().filter(|r| r.action.is_some() || r.move_to.is_some()).count();
        let row = &mut rows[cur];
        let mut next = false;
        match (kind, k.code) {
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => cur = (cur + 1).min(rows.len() - 1),
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => cur = cur.saturating_sub(1),
            (_, KeyCode::Char('?')) => help = true,
            (_, KeyCode::Esc) => return Ok((0, false)),
            (_, KeyCode::Char('q')) => {
                if decided > 0 && !quit_armed {
                    quit_armed = true;
                    note = Some(("unsaved decisions: enter saves, q again quits without them".into(), true));
                    continue;
                }
                return Ok((0, true));
            }
            (Kind::Sort, KeyCode::Char('o')) | (Kind::Mine, KeyCode::Char('o')) | (Kind::News, KeyCode::Char('v')) => {
                opened = if opened == Some(cur) { None } else { Some(cur) }
            }
            (Kind::Sort, KeyCode::Char('n')) => (row.action, next) = (Some("note"), true),
            (Kind::Sort, KeyCode::Char('t')) => {
                row.action = Some("task");
                if row.who.is_none() {
                    row.who = Some(user.to_string());
                }
                next = true;
            }
            (Kind::Sort, KeyCode::Char('x')) | (Kind::News, KeyCode::Char('x')) | (Kind::Mine, KeyCode::Char('x')) => {
                (row.action, next) = (Some("drop"), true)
            }
            (Kind::Sort, KeyCode::Char('w')) => {
                let at = row.who.as_ref().and_then(|w| whos.iter().position(|x| x == w));
                row.who = Some(whos[at.map_or(0, |a| (a + 1) % whos.len())].clone());
                if row.action.is_none() {
                    row.action = Some("task");
                }
            }
            (Kind::Sort, KeyCode::Char('p')) => {
                row.prio = match row.prio {
                    None => Some(1),
                    Some(3) => None,
                    Some(p) => Some(p + 1),
                }
            }
            (Kind::Sort, KeyCode::Char('m')) if !stores.is_empty() => {
                let at = row.move_to.as_ref().and_then(|m| stores.iter().position(|x| x == m));
                row.move_to = match at {
                    None => Some(stores[0].clone()),
                    Some(a) if a + 1 < stores.len() => Some(stores[a + 1].clone()),
                    Some(_) => None,
                };
            }
            (Kind::Sort, KeyCode::Char(c @ ('d' | 'D'))) => {
                let prompt = if c == 'd' { "due, soft target (fri, +3d, 30.11.; - none):" } else { "deadline, hard limit (same forms; - none):" };
                let title = rows[cur].title.clone();
                screen.card.push(Line::from(Span::styled(title, bold())));
                if let Some(text) = ui.input(&mut screen, prompt)? {
                    let value = match text.as_str() {
                        "" => continue,
                        "-" => None,
                        t => match util::parse_date(t) {
                            Ok(d) => Some(d),
                            Err(e) => {
                                note = Some((e, true));
                                continue;
                            }
                        },
                    };
                    let row = &mut rows[cur];
                    if c == 'd' { row.due = value } else { row.deadline = value }
                    if row.action.is_none() {
                        row.action = Some("task");
                        row.who.get_or_insert(user.to_string());
                    }
                }
            }
            (Kind::Sort, KeyCode::Char('A')) => {
                // the row's decision for everything below it
                let (id, r) = (row.id.clone(), row.clone());
                let below: Vec<usize> = rows
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.prefix.contains(&format!("{} › ", r.title)) || x.id.starts_with(&format!("{}/", id)))
                    .map(|(n, _)| n)
                    .collect();
                for n in &below {
                    let x = &mut rows[*n];
                    x.action = r.action;
                    x.who = r.who.clone();
                    x.move_to = r.move_to.clone();
                }
                note = Some((format!("{} rows below took this decision", below.len()), false));
            }
            (Kind::News, KeyCode::Char('o')) => (row.action, next) = (Some("ok"), true),
            (Kind::News, KeyCode::Char('h')) => (row.action, next) = (Some("hold"), true),
            (Kind::Mine, KeyCode::Char('s')) => {
                (row.action, next) = (Some(if row.status == "doing" { "stop" } else { "start" }), true)
            }
            (Kind::Mine, KeyCode::Char('d')) => (row.action, next) = (Some("done"), true),
            (Kind::Mine, KeyCode::Char('l')) => (row.action, next) = (Some("later"), true),
            (_, KeyCode::Char('u')) => {
                row.action = None;
                row.move_to = None;
            }
            (_, KeyCode::Enter) => {
                let n = save(ui, &mut screen, cake, kind, &rows, user)?;
                return Ok((n, false));
            }
            _ => (),
        }
        if next {
            cur = (cur + 1).min(rows.len() - 1);
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
                (Kind::News, Some("hold")) => {
                    cake.call(&["assign", id, "me"])?;
                    if r.status == "doing" {
                        cake.call(&["do", id])?;
                    }
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
                _ => (),
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
            row("step", "name the failed git step", "sync errors unclear › "),
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
            card: render(Kind::Sort, &rows, 4, Some(2), 74, "jul"),
            keys: keys(&[("n/t/x", "note task drop"), ("w", "who"), ("enter", "save")]),
            ..Default::default()
        };
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 22)).unwrap();
        let names: Vec<String> = ["news", "sort", "answer"].iter().map(|s| s.to_string()).collect();
        term.draw(|f| paint(f, &header(&names, 1, 1, 6, 72), &screen, 1, 100)).unwrap();
        let buf = term.backend().buffer().clone();
        let lines: Vec<String> = (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
            .collect();
        println!("{}", lines.join("\n"));
        assert!(lines.iter().any(|l| l.contains("task") && l.contains("renew passport") && l.contains("me") && l.contains("20.11.")));
        assert!(lines.iter().any(|l| l.contains("→paper")));
    }
}
