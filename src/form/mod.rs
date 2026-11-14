//! `cake` / `cake form`: a guided pass over what needs you, made of
//! modules (news, sort, answer, review, mine, dump) run in the
//! configured order or one by one. Every change runs as its own `cake`
//! call, so agents keep working while you think.

mod card;
mod keys;
mod table;

use keys::{Act, Keymap, Text as TextKey};

use crate::{error, markdown, style, theme};
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
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

fn step_color(step: usize) -> Color {
    let t = theme::t();
    let colors = [t.hint, t.heading, t.question, t.ok, t.doing, t.code];
    colors[step % colors.len()]
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


/// what one screen shows, top to bottom in a centered column
#[derive(Default)]
struct Screen {
    tree: Vec<Line<'static>>,
    card_title: String,
    card: Vec<Line<'static>>,
    keys: Line<'static>,
    /// red error, or a neutral line like "saving"
    note: Option<(String, bool)>,
    /// the help overlay: (keys, what), empty when closed
    help: Vec<(String, String)>,
    /// lines pinned under the card text: options, always visible
    choices: Vec<Line<'static>>,
    /// lines pinned at the bottom of the card: the text being typed
    input: Vec<Line<'static>>,
    /// lines the card text is scrolled down (<c-d>, <c-u>)
    scroll: u16,
    /// a card text line to keep visible (the table cursor)
    focus: Option<usize>,
    /// the terminal cursor, columns into the last input line
    cursor: Option<usize>,
    /// a text box over everything: (title, text so far); the cursor follows the text
    overlay: Option<(String, String)>,
}

struct Ui {
    term: DefaultTerminal,
    /// the modules of this run, `step` the current one
    names: Vec<String>,
    /// how many items each module had at the start
    counts: Vec<usize>,
    step: usize,
    handled: usize,
    total: usize,
    /// widest column (config form_width)
    max_width: u16,
    keys: Keymap,
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


/// step names (done ones ✓, the current one big in its color) and the
/// progress bar over all items
fn header(names: &[String], counts: &[usize], step: usize, handled: usize, total: usize, width: u16) -> Vec<Line<'static>> {
        let g = style::g();
        let mut steps = vec![];
        for (n, name) in names.iter().enumerate() {
            if n > 0 {
                steps.push(Span::styled(" · ", dim()));
            }
            // the count says where something waits; dump has none
            let count = counts.get(n).filter(|_| name != "dump").map_or(String::new(), |c| format!(" {}", c));
            steps.push(if n == step {
                Span::styled(format!("{}{}", name.to_uppercase(), count), bold().fg(step_color(n)))
            } else if counts.get(n) == Some(&0) && name != "dump" {
                Span::styled(format!("{}{}", name, count), dim())
            } else {
                Span::styled(format!("{}{}", name, count), Style::default().fg(step_color(n)))
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
        let col_w = column_width(size.width, self.max_width);
        let header_lines = header(&self.names, &self.counts, self.step, self.handled, self.total, col_w.saturating_sub(4));
        let (step, max) = (self.step, self.max_width);
        self.term.draw(|f| paint(f, &header_lines, screen, step, max))?;
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

    /// a line of text in a box over the screen (feedback); None on esc
    fn overlay(&mut self, screen: &mut Screen, title: &str) -> Result<Option<String>, Box<dyn Error>> {
        let keys_before = std::mem::replace(&mut screen.keys, keys(&[("<cr>", "save"), ("<esc> jk", "cancel")]));
        let mut buf = String::new();
        let result = loop {
            screen.overlay = Some((title.to_string(), buf.clone()));
            self.draw(screen)?;
            match self.keys.text()? {
                TextKey::Submit => break Some(buf.trim().to_string()),
                TextKey::Escape => break None,
                TextKey::Backspace => {
                    buf.pop();
                }
                TextKey::Char('\0') => (),
                TextKey::Char(c) => buf.push(c),
            }
        };
        screen.overlay = None;
        screen.keys = keys_before;
        Ok(result)
    }

    /// a line of text inside the card; None on esc
    fn input(&mut self, screen: &mut Screen, prompt: &str) -> Result<Option<String>, Box<dyn Error>> {
        let keys_before = std::mem::replace(&mut screen.keys, keys(&[("<cr>", "save"), ("<esc> jk", "cancel")]));
        let mut buf = String::new();
        // one line like vim: the prompt, then the text, the real cursor after it
        let lead = if prompt == ":" { ":".to_string() } else { format!("{} ", prompt) };
        let hint = if prompt == ":" { "due fri · deadline 30.11. · who ai · prio 1 · move <store> · w · q" } else { "" };
        let result = loop {
            let mut line = vec![Span::styled(lead.clone(), Style::default().fg(theme::t().hint)), Span::raw(buf.clone())];
            if buf.is_empty() && !hint.is_empty() {
                line.push(Span::styled(hint, dim()));
            }
            screen.input = vec![Line::default(), Line::from(line)];
            screen.cursor = Some(lead.chars().count() + buf.chars().count());
            self.draw(screen)?;
            match self.keys.text()? {
                TextKey::Submit => break Some(buf.trim().to_string()),
                TextKey::Escape => break None,
                TextKey::Backspace => {
                    buf.pop();
                }
                TextKey::Char('\0') => (),
                TextKey::Char(c) => buf.push(c),
            }
        };
        screen.input.clear();
        screen.cursor = None;
        screen.keys = keys_before;
        Ok(result)
    }
}

/// the column: the whole width minus a margin, at most `max` (0: no limit)
fn column_width(area: u16, max: u16) -> u16 {
    let w = area.saturating_sub(2).max(area.min(20));
    if max == 0 { w } else { w.min(max) }
}

/// a screen into frame `f`: header box, tree, card (scrolling text,
/// pinned choices and input), keys, note, help
fn paint(f: &mut ratatui::Frame, header_lines: &[Line<'static>], screen: &Screen, step: usize, max: u16) {
    let g = style::g();
    let area = f.area();
    let width = column_width(area.width, max);
    let col = Rect { x: area.x + (area.width - width) / 2, y: area.y, width, height: area.height };
    let inner = width.saturating_sub(4);
    let tree_h = (screen.tree.len() as u16).min(col.height / 3);
    let fixed = 4 + 1 + tree_h + u16::from(tree_h > 0) + 2;
    let avail = col.height.saturating_sub(fixed).max(4);
    let body_h = height(&screen.card, inner);
    let choices_h = height(&screen.choices, inner);
    let input_h = height(&screen.input, inner);
    let sep = u16::from(!screen.choices.is_empty() && body_h > 0);
    let card_h = (body_h + sep + choices_h + input_h + 2).min(avail);
    let [head, _, tree, _, card, keys_a, note_a, _] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(tree_h),
        Constraint::Length(u16::from(tree_h > 0)),
        Constraint::Length(card_h),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(col);
    let block = Block::default().borders(Borders::ALL).border_set(g.border).border_style(dim());
    f.render_widget(Paragraph::new(header_lines.to_vec()).block(block.clone().padding(Padding::horizontal(1))), head);
    f.render_widget(Paragraph::new(screen.tree.clone()), tree);
    // the text scrolls, choices and input stay
    let view = card_h.saturating_sub(2 + sep + choices_h + input_h);
    let max_scroll = body_h.saturating_sub(view);
    let scroll = match screen.focus {
        Some(line) => {
            let before = height(&screen.card[..line.min(screen.card.len())], inner);
            before.saturating_sub(view.saturating_sub(2)).min(max_scroll)
        }
        None => screen.scroll.min(max_scroll),
    };
    let title = Span::styled(format!(" {} ", screen.card_title), bold().fg(step_color(step)));
    let mut card_block = block.clone().title(title).padding(Padding::horizontal(1));
    if max_scroll > 0 {
        let more = match (scroll > 0, scroll < max_scroll) {
            (true, true) => " ↑↓ more <c-u> <c-d> ",
            (false, _) => " ↓ more <c-d> ",
            (true, false) => " ↑ <c-u> ",
        };
        card_block = card_block.title_bottom(Line::from(Span::styled(more, Style::default().fg(theme::t().hint))).right_aligned());
    }
    let inside = card_block.inner(card);
    f.render_widget(card_block, card);
    let [body_a, _, choices_a, input_a] = Layout::vertical([
        Constraint::Length(view),
        Constraint::Length(sep),
        Constraint::Length(choices_h),
        Constraint::Length(input_h),
    ])
    .areas(inside);
    f.render_widget(Paragraph::new(Text::from(screen.card.clone())).wrap(Wrap { trim: false }).scroll((scroll, 0)), body_a);
    f.render_widget(Paragraph::new(Text::from(screen.choices.clone())).wrap(Wrap { trim: false }), choices_a);
    f.render_widget(Paragraph::new(Text::from(screen.input.clone())).wrap(Wrap { trim: false }), input_a);
    if let (Some(at), Some(_)) = (screen.cursor, screen.input.last()) {
        let w = input_a.width.max(1) as usize;
        let above = height(&screen.input[..screen.input.len() - 1], input_a.width);
        let (row, col) = (above as usize + at / w, at % w);
        f.set_cursor_position((input_a.x + col as u16, (input_a.y + row as u16).min(input_a.bottom().saturating_sub(1))));
    }
    f.render_widget(Paragraph::new(screen.keys.clone()), keys_a);
    if let Some((text, error)) = &screen.note {
        let s = if *error { Style::default().fg(theme::t().error) } else { dim() };
        f.render_widget(Paragraph::new(Span::styled(text.clone(), s)), note_a);
    }
    if let Some((title, text)) = &screen.overlay {
        let w = width.min(64).max(20);
        let inner_w = w.saturating_sub(4).max(1) as usize;
        let rows = (text.chars().count() / inner_w + 1) as u16;
        let h = rows + 2;
        let pop = Rect { x: col.x + (width - w) / 2, y: area.y + area.height.saturating_sub(h) / 2, width: w, height: h };
        f.render_widget(Clear, pop);
        let b = block.clone().border_style(Style::default().fg(theme::t().hint)).title(Span::styled(format!(" {} ", title), bold())).padding(Padding::horizontal(1));
        let inside = b.inner(pop);
        f.render_widget(Paragraph::new(text.clone()).wrap(Wrap { trim: false }).block(b), pop);
        let n = text.chars().count();
        f.set_cursor_position((inside.x + (n % inner_w) as u16, inside.y + (n / inner_w) as u16));
    }
    if !screen.help.is_empty() {
        let lines: Vec<Line> = screen
            .help
            .iter()
            .map(|(k, what)| {
                Line::from(vec![
                    Span::styled(format!("{:<18}", k), Style::default().fg(theme::t().hint)),
                    Span::raw(what.to_string()),
                ])
            })
            .collect();
        let h = (lines.len() as u16 + 2).min(area.height);
        let pop = Rect { x: col.x, y: area.y + (area.height - h) / 2, width, height: h };
        f.render_widget(Clear, pop);
        f.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }).block(block.title(Span::styled(" keys ", bold())).padding(Padding::horizontal(1))),
            pop,
        );
    }
}

/// F: feedback on this screen as a task for the AI (tag form-feedback),
/// so it does not wait in the user's own inbox
fn feedback(ui: &mut Ui, screen: &mut Screen, cake: &Cake, store: &str, place: &str) -> Result<(), Box<dyn Error>> {
    if let Some(text) = ui.overlay(screen, "feedback on this screen")?.filter(|t| !t.is_empty()) {
        let body = format!("# {}\n\n- where: {} ({})", text, place, store);
        match ui.save(screen, || cake.call(&["add", "--task", "--assignee", "ai", "-t", "form-feedback", "-m", &body]))? {
            Ok(_) => screen.note = Some(("feedback saved as a task for the AI".into(), false)),
            Err(e) => screen.note = Some((e.to_string(), true)),
        }
    }
    Ok(())
}

/// how a module ended
#[derive(Clone, Copy, PartialEq)]
enum Leave {
    /// everything seen: go on
    Done,
    /// nothing to show: pass it in the direction of travel
    Empty,
    Next,
    Prev,
    Quit,
}

/// what the user did with one item
enum Outcome {
    /// short result for the tree, counted (answered, done, approved)
    Next(Option<String>, bool),
    NextGoal,
    Previous,
    Leave(Leave),
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

/// the modules a form can run, in their default order
pub const MODULES: &[&str] = &["news", "sort", "answer", "review", "mine"];
/// all known modules (dump is optional)
const KNOWN: &[&str] = &["news", "sort", "answer", "review", "mine", "dump"];

/// when the form last ran in a store (for news), in the local state dir
fn last_run_file(root: &Path) -> Option<PathBuf> {
    let base = directories::BaseDirs::new()?;
    let dir = base.state_dir().unwrap_or(base.data_local_dir()).join("cake");
    std::fs::create_dir_all(&dir).ok()?;
    let abs = std::fs::canonicalize(root).unwrap_or(root.to_path_buf());
    Some(dir.join(format!("form{}", abs.to_string_lossy().replace('/', "%"))))
}

fn last_run(root: &Path) -> i64 {
    last_run_file(root).and_then(|f| std::fs::read_to_string(f).ok()).and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

/// a screen without items (empty module, the end): waits for a
/// module move or quit
fn still(ui: &mut Ui, title: &str, lines: Vec<Line<'static>>, fb: (&Cake, &str)) -> Result<Leave, Box<dyn Error>> {
    let k = |a: Act| ui.keys.keys_of(a);
    let parts = [(format!("{}/{}", k(Act::PrevModule), k(Act::NextModule)), "module".to_string()), (k(Act::Quit), "quit".to_string())];
    let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let mut help = false;
    let mut note = None;
    loop {
        let mut screen = Screen {
            card_title: title.to_string(),
            card: lines.clone(),
            keys: keys(&parts),
            help: if help { ui.keys.help() } else { vec![] },
            note: note.take(),
            ..Default::default()
        };
        ui.draw(&screen)?;
        help = false;
        match ui.keys.act()? {
            Act::Feedback => {
                feedback(ui, &mut screen, fb.0, fb.1, title)?;
                note = screen.note.take();
            }
            Act::PrevModule => return Ok(Leave::Prev),
            Act::NextModule => return Ok(Leave::Next),
            Act::Quit => return Ok(Leave::Quit),
            Act::Help => help = true,
            _ => (),
        }
    }
}

/// dump: one new item per line into the inbox of `root`
fn dump(ui: &mut Ui, here: &Cake, store: &str) -> Result<usize, Box<dyn Error>> {
    let mut added: Vec<String> = vec![];
    let mut note = None;
    loop {
        let mut card = vec![
            Line::from("Whatever is on your mind; it goes into the inbox."),
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
            card_title: format!("dump · {}", store),
            card,
            keys: keys(&[("enter", "add / go on"), ("esc", "go on")]),
            note: note.take(),
            ..Default::default()
        };
        match ui.input(&mut screen, "idea:")? {
            Some(text) if !text.is_empty() => match ui.save(&mut screen, || here.call(&["add", "-m", &text]))? {
                Ok(_) => added.push(text),
                Err(e) => note = Some((e.to_string(), true)),
            },
            _ => return Ok(added.len()),
        }
    }
}

pub struct Options {
    /// modules in order
    pub modules: Vec<String>,
    /// all known stores, not only `root`
    pub all: bool,
    /// widest column, 0 none
    pub width: u16,
    /// config `keys`: action -> keys in vim notation
    pub keys: std::collections::BTreeMap<String, String>,
}

/// runs the modules on `root` (with `all`: every known store); dump goes
/// to `root`
pub fn run(root: &Path, user: &str, o: Options) -> Result<(), Box<dyn Error>> {
    if let Some(bad) = o.modules.iter().find(|m| !KNOWN.contains(&m.as_str())) {
        return Err(error::invalid(format!("unknown form module '{}', use {}", bad, KNOWN.join(", "))));
    }
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let known = crate::stores::all()?;
    let name = known.iter().find(|(_, s)| same(s, root)).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut stores: Vec<(String, PathBuf)> = vec![(name, root.to_path_buf())];
    if o.all {
        stores.extend(known.into_iter().filter(|(_, s)| !same(s, root)));
    }
    let since: Vec<i64> = stores.iter().map(|(_, r)| last_run(r)).collect();
    // count first: per module for the header, all for the progress bar
    let mut per_module = vec![0usize; o.modules.len()];
    for ((_, root), since) in stores.iter().zip(&since) {
        let cake = Cake { root };
        for (n, m) in o.modules.iter().enumerate() {
            per_module[n] += match m.as_str() {
                "news" => table::count(&cake, table::Kind::News, user, *since)?,
                "sort" => table::count(&cake, table::Kind::Sort, user, *since)?,
                "mine" => table::count(&cake, table::Kind::Mine, user, *since)?,
                "answer" => card::count(&cake, card::Kind::Answer, user)?,
                "review" => card::count(&cake, card::Kind::Review, user)?,
                _ => 0,
            };
        }
    }
    let total = per_module.iter().sum();
    // nothing to decide (dump always has something to do): no screen at all
    if total == 0 && !o.modules.iter().any(|m| m == "dump") {
        let mut others = 0;
        for (_, r) in &stores {
            let store = crate::store::Store::new_except(r, &[])?;
            others += store.get().iter().filter(|(id, i)| store.is_ready(id) && i.assignee().is_some_and(|a| a != user)).count();
        }
        println!("Nothing waits for you.");
        if others > 0 {
            println!("{} ready task(s) wait for agents or others (cake ready); an agent run or `cake triage` turns them into reviews and questions.", others);
        }
        return Ok(());
    }
    let keymap = Keymap::new(&o.keys)?;
    let mut ui = Ui { term: ratatui::init(), names: o.modules.clone(), counts: per_module, step: 0, handled: 0, total, max_width: o.width, keys: keymap };
    let mut counts = vec![0usize; o.modules.len()];
    let result = (|| -> Result<(), Box<dyn Error>> {
        // h/l move between modules; empty modules are passed in the
        // direction of travel
        let mut n = 0usize;
        let mut back = false;
        let mut walked = false;
        while n < o.modules.len() {
            let m = &o.modules[n];
            ui.step = n;
            let leave = match m.as_str() {
                "dump" => {
                    counts[n] += dump(&mut ui, &Cake { root }, &stores[0].0)?;
                    Leave::Done
                }
                _ => {
                    let mut leave = Leave::Empty;
                    for ((name, sroot), since) in stores.iter().zip(&since) {
                        let cake = Cake { root: sroot };
                        let (c, l) = match m.as_str() {
                            "news" => table::run(&mut ui, &cake, sroot, table::Kind::News, user, name, *since)?,
                            "sort" => table::run(&mut ui, &cake, sroot, table::Kind::Sort, user, name, *since)?,
                            "mine" => table::run(&mut ui, &cake, sroot, table::Kind::Mine, user, name, *since)?,
                            "answer" => card::run(&mut ui, &cake, card::Kind::Answer, user, name)?,
                            "review" => card::run(&mut ui, &cake, card::Kind::Review, user, name)?,
                            _ => (0, Leave::Done),
                        };
                        counts[n] += c;
                        match l {
                            Leave::Done | Leave::Empty => {
                                if l == Leave::Done {
                                    leave = Leave::Done;
                                } else if leave != Leave::Done {
                                    leave = Leave::Empty;
                                }
                            }
                            other => {
                                leave = other;
                                break;
                            }
                        }
                    }
                    leave
                }
            };
            // an empty module is passed when the flow gets there by itself,
            // but shows when the user walks there with h/l
            let leave = if leave == Leave::Empty && walked {
                still(&mut ui, &format!("{} · nothing here", m), vec![Line::from(Span::styled("Nothing waits here right now.", dim()))], (&Cake { root }, &stores[0].0))?
            } else {
                leave
            };
            // after the last module: an end screen instead of closing by accident
            let leave = if n + 1 == o.modules.len() && matches!(leave, Leave::Next | Leave::Done | Leave::Empty) {
                let summary: Vec<String> = o.modules.iter().zip(&counts).filter(|(_, c)| **c > 0).map(|(m, c)| format!("{} {}", m, c)).collect();
                let text = if summary.is_empty() { "Nothing decided this time.".to_string() } else { format!("Decided: {}.", summary.join(", ")) };
                match still(&mut ui, "end of the form", vec![Line::from(text), Line::from(Span::styled("q closes the form, h goes back.", dim()))], (&Cake { root }, &stores[0].0))? {
                    Leave::Prev => Leave::Prev,
                    Leave::Quit => Leave::Quit,
                    _ => continue,
                }
            } else {
                leave
            };
            walked = matches!(leave, Leave::Prev | Leave::Next);
            match leave {
                Leave::Quit => return Ok(()),
                Leave::Prev => {
                    back = true;
                    n = n.saturating_sub(1);
                }
                Leave::Next => {
                    back = false;
                    n += 1;
                }
                // nothing here: keep the direction of travel
                Leave::Empty if back && n > 0 => n -= 1,
                Leave::Empty | Leave::Done => {
                    back = false;
                    n += 1;
                }
            }
        }
        Ok(())
    })();
    let summary: Vec<String> = o.modules.iter().zip(&counts).filter(|(_, c)| **c > 0).map(|(m, c)| format!("{} {}", m, c)).collect();
    let summary = if summary.is_empty() { "nothing waiting".to_string() } else { summary.join(", ") };
    ratatui::restore();
    for (_, r) in &stores {
        if let Some(f) = last_run_file(r) {
            let _ = std::fs::write(f, crate::util::timestamp().to_string());
        }
    }
    result?;
    println!("{}", summary);
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
            term.draw(|f| paint(f, &header(&["news".into(), "sort".into(), "answer".into()], &[0, 2, 3], 2, 2, 5, 52), &screen, 2, 100)).unwrap();
            let buf = term.backend().buffer().clone();
            (0..buf.area.height)
                .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
                .collect::<Vec<_>>()
        };
        let lines = render(None);
        println!("{}", lines.join("\n"));
        let pos = |text: &str| lines.iter().position(|l| l.contains(text)).unwrap_or_else(|| panic!("{} missing", text));
        assert!(pos("ANSWER") < pos("cake CLI") && pos("cake CLI") < pos("by hand") && pos("by hand") < pos("confirm"));
        assert!(lines[pos("Plain ASCII")].contains("a) yes"), "decided items keep their answer");
        let opened = render(Some("sync"));
        println!("{}", opened.join("\n"));
        assert_eq!(opened.iter().position(|l| l.contains("two machines")), Some(opened.iter().position(|l| l.contains("Sync across")).unwrap() + 1), "opened under its line");
    }

    #[test]
    fn command_line_is_one_line_with_the_cursor_after_the_text() {
        let screen = Screen {
            card_title: "sort".into(),
            card: vec![Line::from("a row")],
            input: vec![Line::default(), Line::from(vec![Span::raw(":"), Span::raw("prio 2")])],
            cursor: Some(":prio 2".len()),
            ..Default::default()
        };
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(60, 20)).unwrap();
        let names: Vec<String> = vec!["sort".into()];
        term.draw(|f| paint(f, &header(&names, &[1], 0, 0, 1, 52), &screen, 0, 100)).unwrap();
        let buf = term.backend().buffer().clone();
        let row = (0..buf.area.height)
            .find(|y| (0..buf.area.width).map(|x| buf[(x, *y)].symbol().to_string()).collect::<String>().contains(":prio 2"))
            .expect("one line with :prio 2");
        let line: String = (0..buf.area.width).map(|x| buf[(x, row)].symbol().to_string()).collect();
        let start = line[..line.find(":prio 2").unwrap()].chars().count() as u16;
        let pos = term.get_cursor_position().unwrap();
        assert_eq!((pos.x, pos.y), (start + 7, row), "cursor right after the text");
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

/// writes every screen as text into $CAKE_SCREENS (for the review page):
/// CAKE_SCREENS=dir CAKE_STYLE=ascii cargo test --lib dump_screens -- --ignored
#[cfg(test)]
mod screens {
    use super::*;

    pub(in crate::form) fn dump(name: &str, width: u16, screen: &Screen, step: usize) {
        let Ok(dir) = std::env::var("CAKE_SCREENS") else { return };
        let names: Vec<String> = MODULES.iter().map(|m| m.to_string()).collect();
        let col = column_width(width, 100);
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, 26)).unwrap();
        term.draw(|f| paint(f, &header(&names, &[1, 4, 1, 1, 2, 0], step, 3, 9, col.saturating_sub(4)), screen, step, 100)).unwrap();
        let buf = term.backend().buffer().clone();
        let mut lines: Vec<String> = (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
            .collect();
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        let style = std::env::var("CAKE_STYLE").unwrap_or("unicode".into());
        std::fs::write(format!("{}/{}-{}-{}.txt", dir, name, width, style), lines.join("\n")).unwrap();
    }

    #[test]
    #[ignore]
    fn dump_screens() {
        style::init(&std::env::var("CAKE_STYLE").unwrap_or("unicode".into()));
        let keymap = Keymap::new(&Default::default()).unwrap();
        let item = |id: &str, parent: &str, title: &str, body: &str, status: &str| {
            serde_json::json!({"id": id, "parent": parent, "title": title, "status": status,
                "body": format!("# {}\n\n{}", title, body)})
        };
        let all = vec![
            item("cli", "", "cake CLI: fast capture", "", "task"),
            item("sync", "cli", "Sync errors that say what failed", "- why: unclear git errors", "task"),
            item("hint", "sync", "Hint per error type", "", "task"),
            item("raw", "hint", "Show git's raw message too?", "Git's own text helps experts.\n\n- a) always\n- b) only with -d (recommended)", "task"),
            item("step", "sync", "Name the failed git step", "- changed: fetch, merge, push named\n- try: cake sync offline", "review"),
        ];
        for width in [60u16, 100] {
            let inner = column_width(width, 100).saturating_sub(4);
            // answer card
            let targets: Vec<&Value> = vec![&all[3]];
            let tree = overview(&all, &all[0], &targets);
            let (context, options) = split_options(&body_of(&all[3]));
            let mut card = markdown::render(&context, inner);
            card.push(Line::default());
            for (n, (_, text)) in options.iter().enumerate() {
                let mark = if n == 0 { style::g().select } else { " " };
                card.push(Line::from(format!("{} {}) {}", mark, n + 1, text)));
            }
            card.push(Line::from(format!("  3) {} own words", style::g().ellipsis)));
            let k = |a: Act| keymap.keys_of(a);
            let card_keys = keys(&[("1-3", "choose"), (&k(Act::Confirm), "confirm"), (&k(Act::Insert), "own words"), (&k(Act::Hints), "open"), (&k(Act::Next), "skip"), (&k(Act::Help), "keys")]);
            let screen = Screen {
                tree: tree_lines(&all, &tree, &targets, "raw", &Default::default(), false, Some("sync"), inner),
                card_title: "Show git's raw message too? · cake".into(),
                card,
                keys: card_keys.clone(),
                ..Default::default()
            };
            dump("answer", width, &screen, 2);
            // a long review card with a comment being typed
            let long: String = (1..=18).map(|n| format!("- changed: part {} of the work, with enough words to wrap\n", n)).collect();
            let mut screen = Screen {
                tree: tree_lines(&all, &tree, &[&all[4]], "step", &Default::default(), false, None, inner),
                card_title: "Name the failed git step · cake".into(),
                card: markdown::render(&long, inner),
                choices: vec![Line::from(format!("{} 1) ok, done", style::g().select)), Line::from("  2) drop"), Line::from(format!("  3) {} change something", style::g().ellipsis))],
                keys: card_keys.clone(),
                scroll: 4,
                ..Default::default()
            };
            screen.input = vec![Line::default(), Line::from("what should change:"), Line::from(format!("{} shorter hints please_", style::g().select))];
            dump("review", width, &screen, 3);
            // help overlay
            let screen = Screen { help: keymap.help(), ..Default::default() };
            dump("help", width, &screen, 2);
        }
    }
}
