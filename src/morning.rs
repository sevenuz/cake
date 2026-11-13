//! `cake morning`: a guided pass over what needs you. Every change runs as
//! its own `cake` call, so agents keep working while you think.

use crate::{error, markdown, style, theme};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::DefaultTerminal;
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
    /// agents' questions: an option or own words
    Ask,
    /// work agents finished (tag review): ok, drop or change
    Review,
    /// agents' plans: approve, with dates, drop or a comment
    Plan,
}

const STEPS: [&str; 4] = ["new tasks", "questions", "check", "plans"];

impl Kind {
    fn step(self) -> usize {
        match self {
            Kind::Ask => 1,
            Kind::Review => 2,
            Kind::Plan => 3,
        }
    }
}

fn step_color(step: usize) -> Color {
    let t = theme::t();
    [t.hint, t.question, t.ok, t.heading].get(step).copied().unwrap_or(t.heading)
}

fn dim() -> Style {
    Style::default().fg(theme::t().dim)
}

fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
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

/// what one screen shows, top to bottom in a centered column
#[derive(Default)]
struct Screen {
    tree: Vec<Line<'static>>,
    card_title: String,
    card: Vec<Line<'static>>,
    keys: Line<'static>,
    /// red error, or a neutral line like "saving"
    note: Option<(String, bool)>,
    help: bool,
}

struct Ui {
    term: DefaultTerminal,
    step: usize,
    handled: usize,
    total: usize,
}

/// wrapped height of `lines` at `width`
fn height(lines: &[Line], width: u16) -> u16 {
    let w = width.max(1) as usize;
    lines.iter().map(|l| l.width().max(1).div_ceil(w)).sum::<usize>() as u16
}

fn keys(parts: &[(&str, &str)]) -> Line<'static> {
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

const HELP: &[(&str, &str)] = &[
    ("a-e", "choose an option (j/k or arrows move)"),
    ("enter", "confirm the chosen option"),
    ("i", "own words, a comment or what should change"),
    ("f", "show numbers on the tree, a number opens that item"),
    ("esc", "close an opened item, cancel typing"),
    ("s", "skip this item"),
    ("n", "skip the rest of this goal"),
    ("q", "quit (ctrl-c too), everything chosen is saved"),
    ("?", "this help"),
];

/// step names (done ones ✓, the current one big in its color) and the
/// progress bar over all items
fn header(step: usize, handled: usize, total: usize, width: u16) -> Vec<Line<'static>> {
        let g = style::g();
        let mut steps = vec![];
        for (n, name) in STEPS.iter().enumerate() {
            if n > 0 {
                steps.push(Span::styled(" · ", dim()));
            }
            steps.push(if n < step {
                Span::styled(format!("{} {}", g.check, name), dim())
            } else if n == step {
                Span::styled(name.to_uppercase(), bold().fg(step_color(n)))
            } else {
                Span::styled(name.to_string(), dim())
            });
        }
        let count = format!(" {} / {}", handled, total);
        let bar_w = (width as usize).saturating_sub(count.len()).max(4);
        let full = if total == 0 { 0 } else { bar_w * handled.min(total) / total };
        let bar = vec![
            Span::styled(g.bar_full.repeat(full), Style::default().fg(step_color(step))),
            Span::styled(g.bar_empty.repeat(bar_w - full), dim()),
            Span::styled(count, dim()),
        ];
        vec![Line::from(steps), Line::from(bar)]
}

impl Ui {
    fn draw(&mut self, screen: &Screen) -> Result<(), Box<dyn Error>> {
        let size = self.term.size()?;
        let col_w = size.width.saturating_sub(2).clamp(size.width.min(20), 72);
        let header_lines = header(self.step, self.handled, self.total, col_w.saturating_sub(4));
        let step = self.step;
        self.term.draw(|f| paint(f, &header_lines, screen, step))?;
        Ok(())
    }

    /// one frame with a spinner, then the call
    fn save<T>(&mut self, screen: &mut Screen, call: impl FnOnce() -> T) -> Result<T, Box<dyn Error>> {
        screen.note = Some((format!("{} saving", style::g().spinner[0]), false));
        self.draw(screen)?;
        let result = call();
        screen.note = None;
        Ok(result)
    }

    /// a line of text inside the card; None on esc
    fn input(&mut self, screen: &mut Screen, prompt: &str) -> Result<Option<String>, Box<dyn Error>> {
        let base = screen.card.clone();
        let keys_before = std::mem::replace(&mut screen.keys, keys(&[("enter", "save"), ("esc", "cancel")]));
        let mut buf = String::new();
        let result = loop {
            screen.card = base.clone();
            screen.card.push(Line::default());
            screen.card.push(Line::from(Span::styled(prompt.to_string(), Style::default().fg(theme::t().hint))));
            screen.card.push(Line::from(vec![
                Span::styled(format!("{} ", style::g().select), Style::default().fg(theme::t().hint)),
                Span::raw(buf.clone()),
                Span::styled("_", dim()),
            ]));
            self.draw(screen)?;
            let k = key()?;
            match k.code {
                _ if ctrl_c(&k) => break None,
                KeyCode::Enter => break Some(buf.trim().to_string()),
                KeyCode::Esc => break None,
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char(c) => buf.push(c),
                _ => (),
            }
        };
        screen.card = base;
        screen.keys = keys_before;
        Ok(result)
    }
}

/// a screen into frame `f`: header box, tree, card, keys, note, help
fn paint(f: &mut ratatui::Frame, header_lines: &[Line<'static>], screen: &Screen, step: usize) {
    let g = style::g();
            let area = f.area();
            let width = area.width.saturating_sub(2).clamp(area.width.min(20), 72);
            let col = Rect { x: area.x + (area.width - width) / 2, y: area.y, width, height: area.height };
            let inner = width.saturating_sub(4);
            let tree_h = (screen.tree.len() as u16).min(col.height / 3);
            let card_h = height(&screen.card, inner) + 2;
            let [head, _, tree, gap, card, keys_a, note_a, _] = Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Length(tree_h),
                Constraint::Length(if tree_h > 0 { 1 } else { 0 }),
                Constraint::Max(card_h),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .areas(col);
            let _ = gap;
            let block = Block::default().borders(Borders::ALL).border_set(g.border).border_style(dim());
            f.render_widget(Paragraph::new(header_lines.to_vec()).block(block.clone().padding(Padding::horizontal(1))), head);
            f.render_widget(Paragraph::new(screen.tree.clone()), tree);
            let title = Span::styled(format!(" {} ", screen.card_title), bold().fg(step_color(step)));
            f.render_widget(
                Paragraph::new(Text::from(screen.card.clone()))
                    .wrap(Wrap { trim: false })
                    .block(block.clone().title(title).padding(Padding::horizontal(1))),
                card,
            );
            f.render_widget(Paragraph::new(screen.keys.clone()), keys_a);
            if let Some((text, error)) = &screen.note {
                let s = if *error { Style::default().fg(theme::t().error) } else { dim() };
                f.render_widget(Paragraph::new(Span::styled(text.clone(), s)), note_a);
            }
            if screen.help {
                let lines: Vec<Line> = HELP
                    .iter()
                    .map(|(k, what)| {
                        Line::from(vec![
                            Span::styled(format!("{:<6}", k), Style::default().fg(theme::t().hint)),
                            Span::raw(what.to_string()),
                        ])
                    })
                    .collect();
                let h = (lines.len() as u16 + 2).min(area.height);
                let w = width.min(area.width);
                let pop = Rect { x: col.x, y: area.y + (area.height - h) / 2, width: w, height: h };
                f.render_widget(Clear, pop);
                f.render_widget(
                    Paragraph::new(lines).wrap(Wrap { trim: false }).block(
                        block.title(Span::styled(" keys ", bold())).padding(Padding::horizontal(1)),
                    ),
                    pop,
                );
            }
}

/// what the user did with one item
enum Outcome {
    /// short result for the tree, counted (answered, done, approved)
    Next(Option<String>, bool),
    NextGoal,
    Quit,
}

/// the tree lines of a goal: the current item marked, decided ones with ✓
/// and their result, an opened item's text under its line, hint numbers
fn tree_lines(
    all: &[Value],
    tree: &[(String, String)],
    targets: &[&Value],
    current: &str,
    decided: &std::collections::HashMap<String, String>,
    hints: bool,
    opened: Option<&str>,
    width: u16,
) -> Vec<Line<'static>> {
    let t = theme::t();
    let g = style::g();
    let mut out = vec![];
    for (n, (prefix, id)) in tree.iter().enumerate() {
        let node = find(all, id).unwrap();
        let title = str_of(node, "title").to_string();
        let mut spans = vec![Span::styled(prefix.clone(), dim())];
        if hints {
            let h = HINTS.chars().nth(n).unwrap_or(' ');
            spans.push(Span::styled(format!("{} ", h), Style::default().fg(t.hint).add_modifier(Modifier::BOLD | Modifier::REVERSED)));
        }
        if let Some(result) = decided.get(id) {
            spans.push(Span::styled(format!("{} {}", g.check, title), dim()));
            spans.push(Span::styled(format!("  {}", result), Style::default().fg(t.dim).add_modifier(Modifier::ITALIC)));
        } else if id == current {
            spans.push(Span::styled(format!("? {}", title), bold().fg(t.question)));
            spans.push(Span::styled(" <", Style::default().fg(t.question)));
        } else if targets.iter().any(|x| str_of(x, "id") == id) {
            spans.push(Span::styled(format!("? {}", title), Style::default().fg(t.question)));
        } else if n == 0 {
            spans.push(Span::styled(title, bold()));
        } else {
            spans.push(Span::raw(title));
        }
        out.push(Line::from(spans));
        if opened == Some(id.as_str()) {
            // continue the tree lines through the text
            let cont = match (prefix.strip_suffix(g.branch), prefix.strip_suffix(g.last)) {
                (Some(rest), _) => format!("{}{}", rest, g.pipe),
                (_, Some(rest)) => format!("{}{}", rest, g.space),
                _ => String::new(),
            };
            let indent = format!("{}  ", cont);
            let body = body_of(node);
            let body = if body.is_empty() { "(no text)".to_string() } else { body };
            for l in markdown::render(&body, width.saturating_sub(indent.len() as u16)) {
                let mut spans = vec![Span::styled(indent.clone(), dim())];
                spans.extend(l.spans.into_iter().map(|s| Span::styled(s.content.to_string(), s.style.patch(dim()))));
                out.push(Line::from(spans));
            }
        }
    }
    out
}

/// one item: tree on top, the card with its options below; keys choose,
/// enter confirms
#[allow(clippy::too_many_arguments)]
fn decide(
    ui: &mut Ui,
    cake: &Cake,
    kind: Kind,
    item: &Value,
    all: &[Value],
    goal: &Value,
    targets: &[&Value],
    decided: &std::collections::HashMap<String, String>,
    store: &str,
) -> Result<Outcome, Box<dyn Error>> {
    let t = theme::t();
    let g = style::g();
    let id = str_of(item, "id").to_string();
    let body = body_of(item);
    let (context, mut options) = match kind {
        Kind::Ask => split_options(&body),
        Kind::Plan => (body.clone(), vec![('a', "approve".into()), ('b', "approve with dates".into()), ('c', "drop".into())]),
        Kind::Review => (body.clone(), vec![('a', "ok, done".into()), ('b', "drop".into())]),
    };
    let free = match kind {
        Kind::Ask => "own words",
        Kind::Plan => "comment for the agent",
        Kind::Review => "change something",
    };
    let free_key = (b'a' + options.len() as u8) as char;
    options.push((free_key, format!("{} {}", g.ellipsis, free)));
    let tree = overview(all, goal, targets);
    let width = ui.term.size()?.width.saturating_sub(2).min(72);
    let (mut sel, mut hints, mut help) = (None::<usize>, false, false);
    let mut opened: Option<String> = None;
    let mut note: Option<(String, bool)> = None;
    loop {
        let mut card = markdown::render(&context, width.saturating_sub(4));
        if !card.is_empty() {
            card.push(Line::default());
        }
        for (n, (k, text)) in options.iter().enumerate() {
            let chosen = sel == Some(n);
            let mark = if chosen { g.select } else { " " };
            let s = if chosen { bold().fg(t.hint) } else { Style::default() };
            card.push(Line::from(vec![
                Span::styled(format!("{} {}) ", mark, k), Style::default().fg(t.hint)),
                Span::styled(text.clone(), s),
            ]));
        }
        let key_line = if hints {
            keys(&[("1-9 a-z", "open"), ("esc", "back")])
        } else {
            let mut parts = vec![("a-".to_string() + &free_key.to_string(), "choose".to_string())];
            if sel.is_some() {
                parts.push(("enter".into(), "confirm".into()));
            }
            parts.push(("f".into(), "open".into()));
            if opened.is_some() {
                parts.push(("esc".into(), "close".into()));
            }
            parts.extend([("s".into(), "skip".into()), ("?".into(), "keys".into())]);
            let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            keys(&parts)
        };
        let mut screen = Screen {
            tree: tree_lines(all, &tree, targets, &id, decided, hints, opened.as_deref(), width),
            card_title: format!("{} · {}", str_of(item, "title"), store),
            card,
            keys: key_line,
            note: note.take(),
            help,
        };
        ui.draw(&screen)?;
        let k = key()?;
        if ctrl_c(&k) {
            return Ok(Outcome::Quit);
        }
        if help {
            help = false;
            continue;
        }
        if hints {
            hints = false;
            if let KeyCode::Char(c) = k.code {
                if let Some((_, nid)) = HINTS.find(c).and_then(|n| tree.get(n)) {
                    opened = Some(nid.clone());
                }
            }
            continue;
        }
        let pick = match k.code {
            KeyCode::Esc => {
                opened = None;
                sel = None;
                continue;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = Some(sel.map_or(0, |s| (s + 1).min(options.len() - 1)));
                continue;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                sel = Some(sel.map_or(0, |s| s.saturating_sub(1)));
                continue;
            }
            KeyCode::Char('q') => return Ok(Outcome::Quit),
            KeyCode::Char('n') => return Ok(Outcome::NextGoal),
            KeyCode::Char('s') => return Ok(Outcome::Next(None, false)),
            KeyCode::Char('?') => {
                help = true;
                continue;
            }
            KeyCode::Char('f') => {
                hints = true;
                continue;
            }
            KeyCode::Char('i') => options.len() - 1,
            KeyCode::Char(c) => {
                if let Some(n) = options.iter().position(|(k, _)| *k == c) {
                    sel = Some(n);
                }
                continue;
            }
            KeyCode::Enter => match sel {
                Some(n) => n,
                None => {
                    note = Some(("choose an option first".to_string(), false));
                    continue;
                }
            },
            _ => continue,
        };
        // the chosen option, with text where it needs some
        let last = pick == options.len() - 1;
        let result: Result<(String, bool), Box<dyn Error>> = match (kind, pick) {
            (_, _) if last => {
                let prompt = match kind {
                    Kind::Ask => "your answer:",
                    Kind::Plan => "comment for the agent:",
                    Kind::Review => "what should change:",
                };
                let Some(text) = ui.input(&mut screen, prompt)?.filter(|t| !t.is_empty()) else { continue };
                ui.save(&mut screen, || match kind {
                    Kind::Ask => cake.call(&["done", &id, "-m", &text]).map(|_| (text.clone(), true)),
                    Kind::Plan => cake
                        .call(&["add", "--todo", "-p", &id, "-t", "reply", "-m", &text])
                        .map(|_| ("comment sent".to_string(), false)),
                    Kind::Review => cake
                        .call(&["add", "--todo", "-p", &id, "-t", "reply", "-m", &text])
                        .and_then(|_| cake.call(&["tag", &id, "~review"]))
                        .map(|_| ("sent back".to_string(), false)),
                })?
            }
            (Kind::Ask, n) => {
                let answer = answer_text(&body, &options[n].0.to_string());
                ui.save(&mut screen, || cake.call(&["done", &id, "-m", &answer]).map(|_| (answer.clone(), true)))?
            }
            (Kind::Plan, 0) => ui.save(&mut screen, || cake.call(&["do", &id]).map(|_| ("approved".to_string(), true)))?,
            (Kind::Plan, 1) => {
                let Some(due) = ui.input(&mut screen, "due, soft target (2026-11-30, enter for none):")? else { continue };
                let Some(deadline) = ui.input(&mut screen, "deadline, hard limit (enter for none):")? else { continue };
                ui.save(&mut screen, || {
                    let mut args = vec!["edit", id.as_str()];
                    if !due.is_empty() {
                        args.extend(["--due", &due]);
                    }
                    if !deadline.is_empty() {
                        args.extend(["--deadline", &deadline]);
                    }
                    let edit = if args.len() > 2 { cake.call(&args).map(|_| ()) } else { Ok(()) };
                    edit.and_then(|_| cake.call(&["do", &id])).map(|_| ("approved with dates".to_string(), true))
                })?
            }
            (Kind::Plan, _) => ui.save(&mut screen, || cake.call(&["drop", &id]).map(|_| ("dropped".to_string(), false)))?,
            (Kind::Review, 0) => ui.save(&mut screen, || {
                cake.call(&["tag", &id, "~review"]).and_then(|_| cake.call(&["done", &id])).map(|_| ("done".to_string(), true))
            })?,
            (Kind::Review, _) => ui.save(&mut screen, || {
                cake.call(&["tag", &id, "~review"]).and_then(|_| cake.call(&["drop", &id])).map(|_| ("dropped".to_string(), false))
            })?,
        };
        match result {
            Ok((text, counted)) => {
                let short: String = text.chars().take(40).collect();
                let short = if text.chars().count() > 40 { format!("{}{}", short, g.ellipsis) } else { short };
                return Ok(Outcome::Next(Some(short), counted));
            }
            Err(e) => note = Some((e.to_string(), true)),
        }
    }
}

/// the items of a step in one store and the store's items
fn targets(cake: &Cake, kind: Kind, user: &str) -> Result<(Vec<Value>, Vec<String>), Box<dyn Error>> {
    let all = cake.call(&["ls", "-l"])?;
    let tagged = |i: &Value, tag: &str| i["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == tag));
    let ids: Vec<String> = match kind {
        Kind::Ask => {
            let ready: Vec<String> = cake.call(&["ready", "-t", "ask"])?.iter().map(|i| str_of(i, "id").to_string()).collect();
            all.iter().map(|i| str_of(i, "id").to_string()).filter(|id| ready.contains(id)).collect()
        }
        Kind::Review => all
            .iter()
            .filter(|i| tagged(i, "review") && matches!(str_of(i, "status"), "todo" | "doing"))
            .map(|i| str_of(i, "id").to_string())
            .collect(),
        Kind::Plan => {
            let side = |a: &Value| tagged(a, "ask") || tagged(a, "reply");
            let open_side = |a: &Value| side(a) && matches!(str_of(a, "status"), "todo" | "doing");
            let kids = |i: &Value| -> Vec<&Value> { all.iter().filter(|a| str_of(a, "parent") == str_of(i, "id")).collect() };
            all.iter()
                .filter(|i| i["status"].is_null() && str_of(i, "by") != user && !tagged(i, "doc"))
                .filter(|i| str_of(i, "body").contains("done when"))
                .filter(|i| kids(i).iter().all(|k| side(k)) && !kids(i).iter().any(|k| open_side(k)))
                .map(|i| str_of(i, "id").to_string())
                .collect()
        }
    };
    Ok((all, ids))
}

/// a step in one store, grouped by goal, the most urgent goal first;
/// returns (counted, quit)
fn step(ui: &mut Ui, cake: &Cake, kind: Kind, user: &str, store: &str) -> Result<(usize, bool), Box<dyn Error>> {
    let (all, ids) = targets(cake, kind, user)?;
    let items: Vec<&Value> = ids.iter().filter_map(|id| find(&all, id)).collect();
    let mut goals: Vec<(&Value, Vec<&Value>)> = vec![];
    for it in &items {
        let goal = goal_of(&all, it);
        match goals.iter_mut().find(|(g, _)| str_of(g, "id") == str_of(goal, "id")) {
            Some((_, list)) => list.push(it),
            None => goals.push((goal, vec![it])),
        }
    }
    let score = |v: &&Value| v["score"].as_i64().unwrap_or(0);
    goals.sort_by_key(|(_, list)| std::cmp::Reverse(list.iter().map(score).max().unwrap_or(0)));
    let mut decided = std::collections::HashMap::new();
    let mut counted = 0;
    'goals: for (goal, list) in &goals {
        for (i, it) in list.iter().enumerate() {
            match decide(ui, cake, kind, it, &all, goal, list, &decided, store)? {
                Outcome::Next(result, c) => {
                    ui.handled += 1;
                    counted += c as usize;
                    if let Some(r) = result {
                        decided.insert(str_of(it, "id").to_string(), r);
                    }
                }
                Outcome::NextGoal => {
                    ui.handled += list.len() - i;
                    continue 'goals;
                }
                Outcome::Quit => return Ok((counted, true)),
            }
        }
    }
    Ok((counted, false))
}

/// the cake with `eaten` of its 4 slices gone, from the right
fn cake_art(eaten: usize) -> Vec<Line<'static>> {
    let t = theme::t();
    let n = 4;
    let left = n - eaten.min(n);
    let (mut flame, mut candle, mut top) = (String::from(" "), String::from(" "), String::from(" "));
    for i in 0..n {
        let here = i < left;
        flame += if here { "   ,  " } else { "      " };
        candle += if here { "   |  " } else { "      " };
        top += if here { "______" } else { "      " };
        flame.push(' ');
        candle.push(' ');
        top.push(if here && i + 1 < left { '_' } else { ' ' });
    }
    let row = |fill: &dyn Fn(usize, bool) -> &'static str| {
        let mut s = String::from(if left > 0 { "|" } else { " " });
        for i in 0..n {
            let here = i < left;
            s += fill(i, here);
            s.push(if here || i + 1 < left { '|' } else { ' ' });
        }
        s
    };
    let frost = row(&|_, here| if here { "~~~~~~" } else { "      " });
    let body = row(&|_, _| "      ");
    let base = row(&|i, here| if here { "______" } else if i == left { " . , ." } else { "      " });
    vec![
        Line::from(Span::styled(flame, Style::default().fg(t.doing))),
        Line::from(Span::raw(candle)),
        Line::from(Span::raw(top)),
        Line::from(Span::styled(frost, Style::default().fg(t.code))),
        Line::from(Span::raw(body)),
        Line::from(Span::raw(base)),
        Line::from(Span::styled("=".repeat(n * 7 + 1), dim())),
    ]
}

/// new tasks go to `root`; questions, work to check and plans come from
/// `root`, with `all` from every known store (stores.json, global)
pub fn run(root: &Path, user: &str, all: bool, animations: bool) -> Result<(), Box<dyn Error>> {
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let known = crate::stores::all()?;
    let name = known.iter().find(|(_, s)| same(s, root)).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut stores: Vec<(String, PathBuf)> = vec![(name, root.to_path_buf())];
    if all {
        stores.extend(known.into_iter().filter(|(_, s)| !same(s, root)));
    }
    // count everything first for the progress bar
    let mut total = 0;
    for (_, root) in &stores {
        for kind in [Kind::Ask, Kind::Review, Kind::Plan] {
            total += targets(&Cake { root }, kind, user)?.1.len();
        }
    }
    let mut ui = Ui { term: ratatui::init(), step: 0, handled: 0, total };
    let mut counts = [0usize; 4];
    let mut finished = 0;
    let result = (|| -> Result<(), Box<dyn Error>> {
        let here = Cake { root };
        let mut added: Vec<String> = vec![];
        let mut note = None;
        loop {
            let mut card = vec![
                Line::from("Tasks you already know, e.g. from a paper list."),
                Line::from(Span::styled("Enter adds one, an empty enter goes on.", dim())),
            ];
            if !added.is_empty() {
                card.push(Line::default());
            }
            for a in &added {
                card.push(Line::from(vec![
                    Span::styled(format!("{} ", style::g().check), Style::default().fg(theme::t().ok)),
                    Span::raw(a.clone()),
                ]));
            }
            let mut screen = Screen {
                card_title: format!("new tasks · {}", stores[0].0),
                card,
                keys: keys(&[("enter", "add / go on"), ("esc", "go on")]),
                note: note.take(),
                ..Default::default()
            };
            match ui.input(&mut screen, "task:")? {
                Some(text) if !text.is_empty() => match ui.save(&mut screen, || here.call(&["add", "--todo", "-m", &text]))? {
                    Ok(_) => added.push(text),
                    Err(e) => note = Some((e.to_string(), true)),
                },
                _ => break,
            }
        }
        counts[0] = added.len();
        finished = 1;
        for kind in [Kind::Ask, Kind::Review, Kind::Plan] {
            ui.step = kind.step();
            for (name, root) in &stores {
                let (c, quit) = step(&mut ui, &Cake { root }, kind, user, name)?;
                counts[kind.step()] += c;
                if quit {
                    return Ok(());
                }
            }
            finished += 1;
        }
        Ok(())
    })();
    // the end: a slice per finished step with something in it, then the summary
    let worked = counts.iter().filter(|c| **c > 0).count().min(finished);
    let summary = format!(
        "{} added, {} answered, {} checked, {} approved",
        counts[0], counts[1], counts[2], counts[3]
    );
    let end = (|| -> Result<(), Box<dyn Error>> {
        result.as_ref().map_err(|e| e.to_string())?;
        ui.step = 4;
        let frames = if animations { 0..=worked } else { worked..=worked };
        for eaten in frames {
            let mut card = cake_art(eaten);
            card.push(Line::default());
            card.push(Line::from(Span::styled(
                if ui.total == 0 && counts[0] == 0 { "nothing waiting, enjoy the cake".to_string() } else { summary.clone() },
                bold(),
            )));
            let screen = Screen {
                card_title: "done".into(),
                card,
                keys: keys(&[("any key", "close")]),
                ..Default::default()
            };
            ui.draw(&screen)?;
            if animations && eaten < worked && event::poll(std::time::Duration::from_millis(350))? {
                let _ = event::read()?;
                continue;
            }
        }
        key()?;
        Ok(())
    })();
    ratatui::restore();
    result?;
    end?;
    println!("{}", summary);
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

    /// prints the form as it looks in a 60 column pane (cargo test -- --nocapture)
    #[test]
    fn form_renders_top_to_bottom_in_a_narrow_pane() {
        let item = |id: &str, parent: &str, title: &str, body: &str| {
            serde_json::json!({"id": id, "parent": parent, "title": title, "body": format!("# {}\n\n{}", title, body)})
        };
        let all = vec![
            item("cli", "", "cake CLI: fast capture", ""),
            item("sync", "cli", "Sync across clones", "- why: two machines edit one store"),
            item("rt", "sync", "Nightly routine", ""),
            item("q1", "rt", "How keep the repo list?", "The routine needs to know which repos.\n\n- a) by hand\n- b) stores.json flag"),
            item("q0", "cli", "Plain ASCII output?", "- a) yes\n- b) no"),
        ];
        let targets: Vec<&Value> = vec![&all[4], &all[3]];
        let tree = overview(&all, &all[0], &targets);
        let decided = [("q0".to_string(), "a) yes".to_string())].into();
        let render = |opened: Option<&str>| {
            let screen = Screen {
                tree: tree_lines(&all, &tree, &targets, "q1", &decided, false, opened, 56),
                card_title: "How keep the repo list? · cake".into(),
                card: markdown::render("The routine needs to know which repos.", 52)
                    .into_iter()
                    .chain([Line::default(), Line::from("› a) by hand"), Line::from("  b) stores.json flag")])
                    .collect(),
                keys: keys(&[("a-c", "choose"), ("enter", "confirm"), ("f", "open")]),
                ..Default::default()
            };
            let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(60, 24)).unwrap();
            term.draw(|f| paint(f, &header(1, 2, 5, 52), &screen, 1)).unwrap();
            let buf = term.backend().buffer().clone();
            (0..buf.area.height)
                .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
                .collect::<Vec<_>>()
        };
        let lines = render(None);
        println!("{}", lines.join("\n"));
        let pos = |text: &str| lines.iter().position(|l| l.contains(text)).unwrap_or_else(|| panic!("{} missing", text));
        assert!(pos("QUESTIONS") < pos("cake CLI") && pos("cake CLI") < pos("by hand") && pos("by hand") < pos("confirm"));
        assert!(lines[pos("Plain ASCII")].contains("a) yes"), "decided items keep their answer");
        for eaten in 0..=4 {
            println!("{}", cake_art(eaten).iter().map(|l| l.to_string()).collect::<Vec<_>>().join("\n"));
        }
        let opened = render(Some("sync"));
        println!("{}", opened.join("\n"));
        assert_eq!(opened.iter().position(|l| l.contains("two machines")), Some(opened.iter().position(|l| l.contains("Sync across")).unwrap() + 1), "opened under its line");
    }

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
