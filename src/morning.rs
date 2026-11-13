//! `cake morning`: a guided pass over what needs you. Every change runs as
//! its own `cake` call, so agents keep working while you think.

use crate::{error, markdown, style, theme};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use serde_json::Value;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

/// set for the form's own cake calls: built-ins without alias overrides
pub const PLAIN_ENV: &str = "CAKE_PLAIN";

struct Cake<'a> {
    root: &'a Path,
}

impl Cake<'_> {
    /// runs `cake --store <root> <args> --json`, returns the items
    fn call(&self, args: &[&str]) -> Result<Vec<Value>, Box<dyn Error>> {
        let out = Command::new(exe())
            .arg("--store")
            .arg(self.root)
            .args(args)
            .arg("--json")
            .env(PLAIN_ENV, "1")
            .output()?;
        let v: Value = serde_json::from_slice(&out.stdout)
            .map_err(|_| error::invalid(String::from_utf8_lossy(&out.stderr).trim().to_string()))?;
        if let Some(e) = v.get("error") {
            return Err(error::invalid(e["message"].as_str().unwrap_or("cake failed").to_string()));
        }
        Ok(v["items"].as_array().cloned().unwrap_or_default())
    }
}

/// this binary; after a rebuild while the form runs, Linux names the old
/// one `… (deleted)`, then the new file at that path (or cake on PATH)
fn exe() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    if exe.is_file() {
        return exe;
    }
    let path = exe.to_string_lossy();
    let replaced = std::path::PathBuf::from(path.strip_suffix(" (deleted)").unwrap_or(&path));
    if replaced.is_file() {
        replaced
    } else {
        "cake".into()
    }
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// body without its `# title` line
fn body_of(item: &Value) -> String {
    let body = str_of(item, "body");
    let title = format!("# {}", str_of(item, "title"));
    body.lines().filter(|l| l.trim() != title).collect::<Vec<_>>().join("\n").trim().to_string()
}

/// `a` picks the line `- a) …` of the body, anything else is the answer
fn answer_text(body: &str, input: &str) -> String {
    if input.chars().count() == 1 {
        let tag = format!("- {})", input);
        let lines: Vec<&str> = body.lines().collect();
        if let Some(n) = lines.iter().position(|l| l.trim_start().starts_with(&tag)) {
            // an option may wrap: indented lines below belong to it
            let mut text = lines[n].trim_start().trim_start_matches("- ").to_string();
            for l in &lines[n + 1..] {
                if l.trim().is_empty() || l.trim_start().starts_with("- ") || !l.starts_with(' ') {
                    break;
                }
                text = format!("{} {}", text, l.trim());
            }
            return text;
        }
    }
    input.to_string()
}

fn find<'a>(all: &'a [Value], id: &str) -> Option<&'a Value> {
    all.iter().find(|i| str_of(i, "id") == id)
}

/// the top-level item above `item`
fn goal_of<'a>(all: &'a [Value], item: &'a Value) -> &'a Value {
    let mut cur = item;
    for _ in 0..crate::store::MAX_DEPTH {
        match find(all, str_of(cur, "parent")) {
            Some(p) => cur = p,
            None => break,
        }
    }
    cur
}

/// options `- a) …` of a question (wrapped lines joined) and the rest
fn split_options(body: &str) -> (String, Vec<(char, String)>) {
    let mut context = vec![];
    let mut options: Vec<(char, String)> = vec![];
    for l in body.lines() {
        let t = l.trim_start();
        let mut c = t.chars();
        match (c.next(), c.next(), c.next(), c.next()) {
            (Some('-'), Some(' '), Some(k), Some(')')) if k.is_ascii_lowercase() => {
                options.push((k, t[4..].trim().to_string()));
            }
            _ if !options.is_empty() && l.starts_with(' ') && !t.is_empty() => {
                let last = options.last_mut().unwrap();
                last.1 = format!("{} {}", last.1, t);
            }
            _ => context.push(l),
        }
    }
    (context.join("\n").trim().to_string(), options)
}

/// keys for hints on the overview, like vimium
const HINTS: &str = "123456789abcdeghijklmoprtuvwxyz";

/// the goal's tree, only branches that lead to `targets`: (prefix, id)
fn overview(all: &[Value], goal: &Value, targets: &[&Value]) -> Vec<(String, String)> {
    let mut keep: Vec<&str> = vec![];
    for t in targets {
        let mut cur = Some(*t);
        while let Some(c) = cur {
            if !keep.contains(&str_of(c, "id")) {
                keep.push(str_of(c, "id"));
            }
            cur = find(all, str_of(c, "parent"));
        }
    }
    fn walk(all: &[Value], keep: &[&str], id: &str, prefix: (&str, &str), out: &mut Vec<(String, String)>) {
        let g = style::g();
        out.push((prefix.0.to_string(), id.to_string()));
        let children: Vec<&str> =
            all.iter().filter(|i| str_of(i, "parent") == id && keep.contains(&str_of(i, "id"))).map(|i| str_of(i, "id")).collect();
        for (n, c) in children.iter().enumerate() {
            let last = n + 1 == children.len();
            let own = format!("{}{}", prefix.1, if last { g.last } else { g.branch });
            let below = format!("{}{}", prefix.1, if last { g.space } else { g.pipe });
            walk(all, keep, c, (&own, &below), out);
        }
    }
    let mut out = vec![];
    walk(all, &keep, str_of(goal, "id"), ("", ""), &mut out);
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// agents' questions, answered with an option or own words
    Ask,
    /// agents' plans to approve
    Plan,
    /// work agents finished (tag review): ok, change or drop
    Review,
}

/// what the user did with one item
enum Outcome {
    /// counted: answered or approved
    Next(bool),
    NextGoal,
    Quit,
}

struct Ui {
    term: DefaultTerminal,
    /// green or red line shown with the next screen
    flash: Option<(String, bool)>,
}

fn key() -> Result<KeyEvent, Box<dyn Error>> {
    loop {
        if let Event::Key(k) = event::read()? {
            if k.kind == KeyEventKind::Press {
                return Ok(k);
            }
        }
    }
}

fn ctrl_c(k: &KeyEvent) -> bool {
    k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c')
}

fn dim() -> Style {
    Style::default().fg(theme::t().dim)
}

impl Ui {
    /// draws `body` with a footer: the flash line, then `legend`
    fn draw(&mut self, title: &str, body: &dyn Fn(&mut Frame, Rect), legend: Line) -> Result<(), Box<dyn Error>> {
        let flash = self.flash.clone();
        self.term.draw(|f| {
            let [top, main, foot] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(1), Constraint::Length(2)]).areas(f.area());
            f.render_widget(Paragraph::new(Span::styled(title.to_string(), Style::default().add_modifier(Modifier::BOLD))), top);
            body(f, main);
            let first = match &flash {
                Some((text, ok)) => {
                    Line::from(Span::styled(text.clone(), Style::default().fg(if *ok { theme::t().ok } else { theme::t().error })))
                }
                None => Line::default(),
            };
            f.render_widget(Paragraph::new(vec![first, legend]), foot);
        })?;
        Ok(())
    }

    /// a line of text at the bottom; None on Esc
    fn input(&mut self, title: &str, body: &dyn Fn(&mut Frame, Rect), prompt: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut buf = String::new();
        loop {
            let line = Line::from(vec![
                Span::styled(format!("{} ", prompt), Style::default().fg(theme::t().hint)),
                Span::raw(buf.clone()),
                Span::styled("_", dim()),
            ]);
            self.draw(title, body, line)?;
            let k = key()?;
            match k.code {
                _ if ctrl_c(&k) => return Ok(None),
                KeyCode::Enter => return Ok(Some(buf.trim().to_string())),
                KeyCode::Esc => return Ok(None),
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char(c) => buf.push(c),
                _ => (),
            }
        }
    }
}

fn legend(parts: &[(&str, &str)]) -> Line<'static> {
    let mut spans = vec![];
    for (n, (k, what)) in parts.iter().enumerate() {
        if n > 0 {
            spans.push(Span::styled("  ", dim()));
        }
        spans.push(Span::styled(k.to_string(), Style::default().fg(theme::t().hint)));
        spans.push(Span::styled(format!(" {}", what), dim()));
    }
    Line::from(spans)
}

/// one item: overview on top, the item below, an opened item beside it
#[allow(clippy::too_many_arguments)]
fn decide(
    ui: &mut Ui,
    cake: &Cake,
    kind: Kind,
    item: &Value,
    all: &[Value],
    goal: &Value,
    targets: &[&Value],
    title: &str,
) -> Result<Outcome, Box<dyn Error>> {
    let t = theme::t();
    let id = str_of(item, "id").to_string();
    let body = body_of(item);
    let (context, options) = match kind {
        Kind::Ask => split_options(&body),
        Kind::Plan => (
            body.clone(),
            vec![('a', "approve".to_string()), ('b', "approve with dates".to_string()), ('c', "drop".to_string())],
        ),
        Kind::Review => (
            body.clone(),
            vec![('a', "ok, done".to_string()), ('b', "change something".to_string()), ('c', "drop".to_string())],
        ),
    };
    let tree = overview(all, goal, targets);
    let mut hints = false;
    let mut opened: Option<String> = None;
    loop {
        let render = |f: &mut Frame, area: Rect| {
            let tree_h = (tree.len() as u16 + 1).min(area.height / 3).max(2);
            let [top, rest] = Layout::vertical([Constraint::Length(tree_h), Constraint::Min(1)]).areas(area);
            let lines: Vec<Line> = tree
                .iter()
                .enumerate()
                .map(|(n, (prefix, nid))| {
                    let node = find(all, nid).unwrap();
                    let mut spans = vec![Span::styled(prefix.clone(), dim())];
                    if hints {
                        let h = HINTS.chars().nth(n).map_or(' ', |c| c);
                        spans.push(Span::styled(format!("[{}] ", h), Style::default().fg(t.hint).add_modifier(Modifier::BOLD)));
                    }
                    let is_target = targets.iter().any(|x| str_of(x, "id") == nid);
                    let text = str_of(node, "title").to_string();
                    spans.push(if *nid == id {
                        Span::styled(format!("? {}", text), Style::default().fg(t.question).add_modifier(Modifier::BOLD))
                    } else if is_target {
                        Span::styled(format!("? {}", text), Style::default().fg(t.question))
                    } else if n == 0 {
                        Span::styled(text, Style::default().add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw(text)
                    });
                    Line::from(spans)
                })
                .collect();
            f.render_widget(Paragraph::new(lines), top);
            let (main, side) = match &opened {
                Some(_) => {
                    let [a, b] = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(rest);
                    (a, Some(b))
                }
                None => (rest, None),
            };
            let mut text: Vec<Line> = vec![Line::from(Span::styled(
                str_of(item, "title").to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ))];
            text.push(Line::default());
            text.extend(markdown::render(&context, main.width));
            text.push(Line::default());
            for (k, o) in &options {
                text.push(Line::from(vec![
                    Span::styled(format!("  {}) ", k), Style::default().fg(t.hint).add_modifier(Modifier::BOLD)),
                    Span::raw(o.clone()),
                ]));
            }
            f.render_widget(Paragraph::new(Text::from(text)).wrap(Wrap { trim: false }), main);
            if let (Some(side), Some(oid)) = (side, &opened) {
                let node = find(all, oid).unwrap();
                let lines = markdown::render(&body_of(node), side.width.saturating_sub(2));
                let block = Block::default().borders(Borders::LEFT).border_style(dim()).title(Span::styled(
                    format!(" {}", str_of(node, "title")),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }).block(block), side);
            }
        };
        let keys = match (kind, hints) {
            (_, true) => legend(&[("1-9 a-z", "open"), ("esc", "back")]),
            (Kind::Ask, false) => {
                let range = match options.len() {
                    0 => String::new(),
                    1 => "a".to_string(),
                    n => format!("a-{}", options[n - 1].0),
                };
                let mut parts = vec![];
                if !range.is_empty() {
                    parts.push((range.as_str(), "choose"));
                }
                parts.extend([("i", "own words"), ("s", "skip"), ("f", "open"), ("n", "next goal"), ("q", "quit")]);
                legend(&parts)
            }
            (Kind::Plan, false) => legend(&[("a-c", "choose"), ("i", "comment"), ("s", "skip"), ("f", "open"), ("n", "next goal"), ("q", "quit")]),
            (Kind::Review, false) => legend(&[("a-c", "choose"), ("s", "skip"), ("f", "open"), ("n", "next goal"), ("q", "quit")]),
        };
        let keys = if opened.is_some() && !hints {
            let mut l = keys;
            l.spans.push(Span::styled("  esc", Style::default().fg(t.hint)));
            l.spans.push(Span::styled(" close", dim()));
            l
        } else {
            keys
        };
        ui.draw(title, &render, keys)?;
        let k = key()?;
        if ctrl_c(&k) {
            return Ok(Outcome::Quit);
        }
        let KeyCode::Char(c) = k.code else {
            if k.code == KeyCode::Esc {
                hints = false;
                opened = None;
            }
            continue;
        };
        if hints {
            hints = false;
            if let Some(n) = HINTS.find(c) {
                if let Some((_, nid)) = tree.get(n) {
                    opened = Some(nid.clone());
                }
            }
            continue;
        }
        ui.flash = None;
        let result: Result<(String, bool), Box<dyn Error>> = match (kind, c) {
            (_, 'q') => return Ok(Outcome::Quit),
            (_, 'n') => return Ok(Outcome::NextGoal),
            (_, 's') => return Ok(Outcome::Next(false)),
            (_, 'f') => {
                hints = true;
                continue;
            }
            (Kind::Ask, 'i') => match ui.input(title, &render, "your answer:")? {
                Some(text) if !text.is_empty() => cake.call(&["done", &id, "-m", &text]).map(|_| (format!("answered: {}", text), true)),
                _ => continue,
            },
            (Kind::Plan, 'i') => match ui.input(title, &render, "comment for the agent:")? {
                Some(text) if !text.is_empty() => cake
                    .call(&["add", "--todo", "-p", &id, "-t", "reply", "-m", &text])
                    .map(|_| ("comment saved, the agent reworks the plan".to_string(), false)),
                _ => continue,
            },
            (Kind::Ask, c) if options.iter().any(|(k, _)| *k == c) => {
                let answer = answer_text(&body, &c.to_string());
                cake.call(&["done", &id, "-m", &answer]).map(|_| (format!("answered: {}", answer), true))
            }
            (Kind::Plan, 'a') => cake.call(&["do", &id]).map(|_| ("approved".to_string(), true)),
            (Kind::Plan, 'b') => {
                let Some(due) = ui.input(title, &render, "due, soft target (2026-11-30, enter none):")? else { continue };
                let Some(deadline) = ui.input(title, &render, "deadline, hard limit (enter none):")? else { continue };
                let mut args = vec!["edit", id.as_str()];
                if !due.is_empty() {
                    args.extend(["--due", &due]);
                }
                if !deadline.is_empty() {
                    args.extend(["--deadline", &deadline]);
                }
                let edit = if args.len() > 2 { cake.call(&args).map(|_| ()) } else { Ok(()) };
                edit.and_then(|_| cake.call(&["do", &id])).map(|_| ("approved with dates".to_string(), true))
            }
            (Kind::Plan, 'c') => cake.call(&["drop", &id]).map(|_| ("dropped".to_string(), false)),
            (Kind::Review, 'a') => cake
                .call(&["tag", &id, "~review"])
                .and_then(|_| cake.call(&["done", &id]))
                .map(|_| ("done".to_string(), true)),
            (Kind::Review, 'b') => match ui.input(title, &render, "what should change:")? {
                Some(text) if !text.is_empty() => cake
                    .call(&["add", "--todo", "-p", &id, "-t", "reply", "-m", &text])
                    .and_then(|_| cake.call(&["tag", &id, "~review"]))
                    .map(|_| ("sent back to the agent".to_string(), false)),
                _ => continue,
            },
            (Kind::Review, 'c') => cake
                .call(&["tag", &id, "~review"])
                .and_then(|_| cake.call(&["drop", &id]))
                .map(|_| ("dropped".to_string(), false)),
            _ => continue,
        };
        match result {
            Ok((text, counted)) => {
                ui.flash = Some((text, true));
                return Ok(Outcome::Next(counted));
            }
            Err(e) => ui.flash = Some((e.to_string(), false)),
        }
    }
}

/// the items of a step in one store, grouped by goal, the goal with the
/// most urgent item first; returns (counted, shown, quit)
fn step(ui: &mut Ui, cake: &Cake, kind: Kind, user: &str, name: &str) -> Result<(usize, usize, bool), Box<dyn Error>> {
    let all = cake.call(&["ls", "-l"])?;
    let targets: Vec<&Value> = match kind {
        Kind::Ask => {
            let ids: Vec<String> = cake.call(&["ready", "-t", "ask"])?.iter().map(|i| str_of(i, "id").to_string()).collect();
            all.iter().filter(|i| ids.iter().any(|id| id == str_of(i, "id"))).collect()
        }
        Kind::Review => {
            let ids: Vec<String> =
                cake.call(&["ls", "-t", "review", "--status", "todo,doing"])?.iter().map(|i| str_of(i, "id").to_string()).collect();
            // ls also lists the children of matches: keep the tagged ones
            let tagged = |i: &Value| i["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "review"));
            all.iter().filter(|i| tagged(i) && ids.iter().any(|id| id == str_of(i, "id"))).collect()
        }
        Kind::Plan => {
            let side = |a: &Value| a["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "ask" || t == "reply"));
            let open_side = |a: &Value| side(a) && matches!(str_of(a, "status"), "todo" | "doing");
            let kids = |i: &Value| -> Vec<&Value> { all.iter().filter(|a| str_of(a, "parent") == str_of(i, "id")).collect() };
            all.iter()
                .filter(|i| i["status"].is_null() && str_of(i, "by") != user)
                .filter(|i| !i["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "doc")))
                .filter(|i| str_of(i, "body").contains("done when"))
                .filter(|i| kids(i).iter().all(|k| side(k)) && !kids(i).iter().any(|k| open_side(k)))
                .collect()
        }
    };
    let mut goals: Vec<(&Value, Vec<&Value>)> = vec![];
    for t in &targets {
        let goal = goal_of(&all, t);
        match goals.iter_mut().find(|(g, _)| str_of(g, "id") == str_of(goal, "id")) {
            Some((_, items)) => items.push(t),
            None => goals.push((goal, vec![t])),
        }
    }
    let score = |v: &&Value| v["score"].as_i64().unwrap_or(0);
    goals.sort_by_key(|(_, items)| std::cmp::Reverse(items.iter().map(score).max().unwrap_or(0)));
    let what = match kind {
        Kind::Ask => "Questions",
        Kind::Review => "Finished, please check",
        Kind::Plan => "Plans to approve",
    };
    let (mut counted, mut n) = (0, 0);
    'goals: for (goal, items) in &goals {
        for (i, item) in items.iter().enumerate() {
            n += 1;
            let title = format!("{} {}/{}  {}", what, n, targets.len(), name);
            match decide(ui, cake, kind, item, &all, goal, items, &title)? {
                Outcome::Next(c) => counted += c as usize,
                Outcome::NextGoal => {
                    n += items.len() - i - 1;
                    continue 'goals;
                }
                Outcome::Quit => return Ok((counted, targets.len(), true)),
            }
        }
    }
    Ok((counted, targets.len(), false))
}

/// new tasks go to `root`; questions and plans come from `root`, with
/// `all` from every known store (stores.json, global)
pub fn run(root: &Path, user: &str, all: bool) -> Result<(), Box<dyn Error>> {
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let known = crate::stores::all()?;
    let name = known.iter().find(|(_, s)| same(s, root)).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut stores: Vec<(String, PathBuf)> = vec![(name, root.to_path_buf())];
    if all {
        stores.extend(known.into_iter().filter(|(_, s)| !same(s, root)));
    }
    let mut ui = Ui { term: ratatui::init(), flash: None };
    let result = (|| -> Result<(usize, usize, usize, usize), Box<dyn Error>> {
        let here = Cake { root };
        let mut added: Vec<String> = vec![];
        let title = format!("New tasks  {}", stores[0].0);
        loop {
            let list = added.clone();
            let body = |f: &mut Frame, area: Rect| {
                let mut lines = vec![
                    Line::from("Tasks you already know, e.g. from a paper list. Enter adds, empty enter or esc goes on."),
                    Line::default(),
                ];
                lines.extend(list.iter().map(|t| Line::from(vec![Span::styled(format!("{} ", style::g().todo), dim()), Span::raw(t.clone())])));
                f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
            };
            match ui.input(&title, &body, "task:")? {
                Some(text) if !text.is_empty() => match here.call(&["add", "--todo", "-m", &text]) {
                    Ok(_) => {
                        added.push(text);
                        ui.flash = None;
                    }
                    Err(e) => ui.flash = Some((e.to_string(), false)),
                },
                _ => break,
            }
        }
        let (mut answered, mut approved, mut reviewed) = (0, 0, 0);
        'steps: for kind in [Kind::Ask, Kind::Review, Kind::Plan] {
            for (name, root) in &stores {
                let (counted, _, quit) = step(&mut ui, &Cake { root }, kind, user, name)?;
                match kind {
                    Kind::Ask => answered += counted,
                    Kind::Review => reviewed += counted,
                    Kind::Plan => approved += counted,
                }
                if quit {
                    break 'steps;
                }
            }
        }
        Ok((added.len(), answered, reviewed, approved))
    })();
    let last_flash = ui.flash.take();
    ratatui::restore();
    let (added, answered, reviewed, approved) = result?;
    if let Some((text, ok)) = last_flash {
        println!("{}", theme::fg(&text, if ok { theme::t().ok } else { theme::t().error }));
    }
    println!("{} added, {} answered, {} checked, {} approved", added, answered, reviewed, approved);
    for (name, root) in &stores {
        let ready = Cake { root }.call(&["ready", "-t", "~ask,~reply"])?;
        let agents = ready.iter().filter(|i| str_of(i, "by") != user).count();
        if !ready.is_empty() {
            println!("  {}: {} ready for agents, {} for you", name, agents, ready.len() - agents);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_letter_picks_the_option_line() {
        let body = "Context.\n\n- a) sqlite (recommended)\n- b) files";
        assert_eq!(answer_text(body, "b"), "b) files");
        let wrapped = "- a) by hand (recommended:\n  no code)\n- b) flag";
        assert_eq!(answer_text(wrapped, "a"), "a) by hand (recommended: no code)");
        assert_eq!(answer_text(body, "c"), "c");
        assert_eq!(answer_text(body, "files, but plain"), "files, but plain");
    }
}
