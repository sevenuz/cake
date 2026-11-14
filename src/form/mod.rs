//! `cake` / `cake form`: a guided pass over what needs you, made of
//! modules (news, inbox, answer, review, mine, dump) run in the
//! configured order or one by one. Every change runs as its own `cake`
//! call, so agents keep working while you think.

mod keys;
mod list;
mod tree;

use keys::{Act, Keymap, Text as TextKey};

use crate::{error, markdown, style, theme};
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

/// `s` padded to `w` columns, or cut with an ellipsis
fn cut(s: &str, w: usize) -> String {
    if s.chars().count() <= w {
        return format!("{:<w$}", s, w = w);
    }
    let g = style::g();
    let keep = w.saturating_sub(g.ellipsis.chars().count());
    format!("{}{}", s.chars().take(keep).collect::<String>(), g.ellipsis)
}

fn find<'a>(all: &'a [Value], id: &str) -> Option<&'a Value> {
    all.iter().find(|i| str_of(i, "id") == id)
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
    /// tree modules: the tree line of the cursor; the tree fills the screen
    tree_focus: Option<usize>,
    /// the card has the focus: its border in the hint color
    card_active: bool,
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
    /// parents shown above an item (config tree_depth)
    tree_depth: usize,
    /// who a task from the inbox module gets (config sort_who)
    sort_who: String,
    /// go on after the last open item (config form_auto_next)
    auto_next: bool,
    /// items changed this run per store: they stay in their module
    changed: std::collections::HashMap<PathBuf, std::collections::HashSet<String>>,
    /// how changed items were before, for undo
    before: std::collections::HashMap<(PathBuf, String), list::Before>,
    /// what a change gave per item (an answer, "sent back"), shown in the tree
    results: std::collections::HashMap<(PathBuf, String), String>,
}

/// wrapped height of `lines` at `width`
fn height(lines: &[Line], width: u16) -> u16 {
    let w = width.max(1) as usize;
    lines.iter().map(|l| l.width().max(1).div_ceil(w)).sum::<usize>() as u16
}

/// word wrap one line at `w` columns, the same way it is drawn; returns
/// the rows and the (row, column) of char `at` (the end: after the last char)
fn wrap_line(line: &Line<'static>, w: usize, at: usize) -> (Vec<Line<'static>>, (usize, usize)) {
    let w = w.max(1);
    let chars: Vec<(char, Style)> = line.spans.iter().flat_map(|s| s.content.chars().map(move |c| (c, s.style))).collect();
    let cw = |c: char| Span::raw(c.to_string()).width();
    let mut rows: Vec<Vec<(char, Style)>> = vec![vec![]];
    let mut row_w = 0;
    let mut pos = (0, 0);
    for (i, &(c, st)) in chars.iter().enumerate() {
        let starts_word = c != ' ' && (i == 0 || chars[i - 1].0 == ' ');
        let word_w: usize = if starts_word { chars[i..].iter().take_while(|(c, _)| *c != ' ').map(|(c, _)| cw(*c)).sum() } else { 0 };
        if row_w > 0 && (row_w + cw(c) > w || (starts_word && word_w <= w && row_w + word_w > w)) {
            rows.push(vec![]);
            row_w = 0;
        }
        if i == at {
            pos = (rows.len() - 1, row_w);
        }
        rows.last_mut().unwrap().push((c, st));
        row_w += cw(c);
    }
    if at >= chars.len() {
        pos = if row_w >= w { (rows.len(), 0) } else { (rows.len() - 1, row_w) };
        if row_w >= w {
            rows.push(vec![]);
        }
    }
    let lines = rows
        .into_iter()
        .map(|r| Line::from(r.into_iter().map(|(c, st)| Span::styled(c.to_string(), st)).collect::<Vec<_>>()))
        .collect();
    (lines, pos)
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
    let body_h = height(&screen.card, inner);
    let choices_h = height(&screen.choices, inner);
    // the input line wrapped here, so text and cursor agree
    let mut input = screen.input.clone();
    let mut input_cursor = None;
    if let Some(last) = input.pop() {
        let (rows, (r, c)) = wrap_line(&last, inner as usize, screen.cursor.unwrap_or(usize::MAX));
        let above = height(&input, inner);
        input.extend(rows);
        input_cursor = screen.cursor.map(|_| (above as usize + r, c));
    }
    let input_h = height(&input, inner);
    let sep = u16::from(!screen.choices.is_empty() && body_h > 0);
    let has_card = !screen.card_title.is_empty() || !screen.card.is_empty() || !screen.choices.is_empty() || !screen.input.is_empty();
    let natural = if has_card { body_h + sep + choices_h + input_h + 2 } else { 0 };
    let (tree_h, tree_scroll, card_h) = match screen.tree_focus {
        // tree modules: the tree takes the room, a card at most half of it
        Some(focus) => {
            // the tree takes what it needs up to half, the box the rest
            let avail = col.height.saturating_sub(4 + 1 + 2);
            let len = screen.tree.len() as u16;
            let gap = u16::from(has_card);
            let tree_h = len.min((avail / 2).max(avail.saturating_sub(natural + gap)));
            let card_h = natural.min(avail.saturating_sub(tree_h + gap));
            // the cursor row and one line under it stay in view
            let scroll = (focus as u16 + 2).saturating_sub(tree_h).min(len.saturating_sub(tree_h));
            (tree_h, scroll, card_h)
        }
        None => {
            let tree_h = (screen.tree.len() as u16).min(col.height / 3);
            let fixed = 4 + 1 + tree_h + u16::from(tree_h > 0) + 2;
            let avail = col.height.saturating_sub(fixed).max(4);
            (tree_h, 0, natural.min(avail))
        }
    };
    let [head, _, tree, _, card, keys_a, note_a, _] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(tree_h),
        Constraint::Length(u16::from(tree_h > 0 && card_h > 0)),
        Constraint::Length(card_h),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(col);
    let block = Block::default().borders(Borders::ALL).border_set(g.border).border_style(dim());
    f.render_widget(Paragraph::new(header_lines.to_vec()).block(block.clone().padding(Padding::horizontal(1))), head);
    f.render_widget(Paragraph::new(screen.tree.clone()).scroll((tree_scroll, 0)), tree);
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
    if screen.card_active {
        card_block = card_block.border_style(Style::default().fg(theme::t().hint));
    }
    let inside = card_block.inner(card);
    if card_h > 0 {
        f.render_widget(card_block, card);
    }
    let [body_a, _, choices_a, input_a] = Layout::vertical([
        Constraint::Length(view),
        Constraint::Length(sep),
        Constraint::Length(choices_h),
        Constraint::Length(input_h),
    ])
    .areas(inside);
    f.render_widget(Paragraph::new(Text::from(screen.card.clone())).wrap(Wrap { trim: false }).scroll((scroll, 0)), body_a);
    f.render_widget(Paragraph::new(Text::from(screen.choices.clone())).wrap(Wrap { trim: false }), choices_a);
    f.render_widget(Paragraph::new(Text::from(input)).wrap(Wrap { trim: false }), input_a);
    if let Some((row, col)) = input_cursor {
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
        let (lines, (crow, ccol)) = wrap_line(&Line::from(text.clone()), inner_w, usize::MAX);
        let h = (lines.len() as u16 + 2).min(area.height);
        let pop = Rect { x: col.x + (width - w) / 2, y: area.y + area.height.saturating_sub(h) / 2, width: w, height: h };
        f.render_widget(Clear, pop);
        let b = block.clone().border_style(Style::default().fg(theme::t().hint)).title(Span::styled(format!(" {} ", title), bold())).padding(Padding::horizontal(1));
        let inside = b.inner(pop);
        f.render_widget(Paragraph::new(lines).block(b), pop);
        f.set_cursor_position((inside.x + ccol as u16, (inside.y + crow as u16).min(inside.bottom().saturating_sub(1))));
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


/// the modules a form can run, in their default order
pub const MODULES: &[&str] = &["news", "inbox", "answer", "review", "mine"];
/// all known modules (dump is optional)
const KNOWN: &[&str] = &["news", "inbox", "answer", "review", "mine", "dump"];

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
    /// parents shown above an item
    pub tree_depth: usize,
    /// who a task from the inbox module gets: me, ai, a name, - nobody
    pub sort_who: String,
    /// after the last open item of a module, go on
    pub auto_next: bool,
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
            per_module[n] += match list::Kind::of(m) {
                Some(kind) => list::count(&cake, kind, user, *since)?,
                None => 0,
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
    let mut ui = Ui { term: ratatui::init(), names: o.modules.clone(), counts: per_module, step: 0, handled: 0, total, max_width: o.width, keys: keymap, tree_depth: o.tree_depth, sort_who: o.sort_who.clone(), auto_next: o.auto_next, changed: Default::default(), before: Default::default(), results: Default::default() };
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
                        let (c, l) = match list::Kind::of(m) {
                            Some(kind) => list::run(&mut ui, &cake, sroot, kind, user, name, *since)?,
                            None => (0, Leave::Done),
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

    fn render(screen: &Screen, w: u16, h: u16) -> Vec<String> {
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
        term.draw(|f| paint(f, &header(&["news".into(), "sort".into(), "answer".into()], &[0, 2, 3], 2, 2, 5, w - 8), screen, 2, 100)).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area.height).map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string()).collect()
    }

    /// tree modules: the tree fills the screen, the card under it, the
    /// cursor row stays in view (cargo test -- --nocapture prints it)
    #[test]
    fn tree_fills_the_screen_and_follows_the_cursor() {
        let tree: Vec<Line> = (0..40).map(|n| Line::from(format!("row {}", n))).collect();
        let screen = Screen {
            tree: tree.clone(),
            tree_focus: Some(2),
            card_title: "How keep the repo list? · cake".into(),
            card: vec![Line::from("The routine needs to know which repos.")],
            choices: vec![Line::from("› 1) by hand"), Line::from("  2) stores.json flag")],
            keys: keys(&[("<cr>", "answer")]),
            ..Default::default()
        };
        let lines = render(&screen, 60, 30);
        println!("{}", lines.join("\n"));
        let pos = |text: &str| lines.iter().position(|l| l.contains(text)).unwrap_or_else(|| panic!("{} missing", text));
        assert!(pos("ANSWER") < pos("row 0") && pos("row 0") < pos("by hand") && pos("by hand") < pos("answer"));
        assert!(!lines[pos("row 0")].contains('│'), "the tree has no box");
        let lines = render(&Screen { tree_focus: Some(35), ..screen }, 60, 30);
        assert!(lines.iter().any(|l| l.trim() == "row 35") && lines.iter().any(|l| l.trim() == "row 36"), "cursor and the row under it in view");
        // no card: the tree takes the whole height
        let lines = render(&Screen { tree, tree_focus: Some(0), ..Default::default() }, 60, 30);
        assert!(lines.iter().any(|l| l.trim() == "row 21"));
    }

    #[test]
    fn wrap_breaks_at_words_and_the_cursor_follows() {
        let text = |ls: &[Line]| ls.iter().map(|l| l.to_string()).collect::<Vec<_>>();
        let (ls, pos) = wrap_line(&Line::from("aaa bbb cc"), 8, usize::MAX);
        assert_eq!(text(&ls), ["aaa bbb ", "cc"]);
        assert_eq!(pos, (1, 2), "after the last char");
        let (ls, pos) = wrap_line(&Line::from("aaa bbbb"), 8, usize::MAX);
        assert_eq!(text(&ls), ["aaa bbbb", ""], "full row: cursor on the next");
        assert_eq!(pos, (1, 0));
        let (ls, pos) = wrap_line(&Line::from("x abcdefghijk"), 5, usize::MAX);
        assert_eq!(text(&ls), ["x abc", "defgh", "ijk"], "a long word is cut");
        assert_eq!(pos, (2, 3));
        let (ls, pos) = wrap_line(&Line::from(vec![Span::raw("> "), Span::raw("hint text")]), 6, 2);
        assert_eq!(text(&ls), ["> hint", " text"]);
        assert_eq!(pos, (0, 2), "cursor before the hint");
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
        for width in [60u16, 100] {
            // help overlay
            let screen = Screen { help: keymap.help(), ..Default::default() };
            dump("help", width, &screen, 2);
        }
    }
}
