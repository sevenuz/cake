//! the dashboard on bare `cake`: one tab per store (and `all`), what runs,
//! where to put your focus next (what unblocks agents first) and what the
//! agents do next; a card for the selected line; <cr> opens the form on
//! that module and store, back here afterwards. Re-reads every 2 s.

use crate::commands::reviews_full;
use crate::config::{Config, StoreConfig};
use crate::item::{Item, Status};
use crate::stats::{self, Period};
use crate::store::Store;
use crate::{markdown, style, theme, util};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Wrap};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// ai queue lines per store
const QUEUE: usize = 5;
/// own tasks in the focus list per store
const OWN: usize = 3;

struct Src {
    name: String,
    root: PathBuf,
    store: Store,
    config: StoreConfig,
}

/// all known stores, `here` first (also when it is not registered)
fn load(here: &Path) -> Result<Vec<Src>, Box<dyn Error>> {
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let known = crate::stores::all().unwrap_or_default();
    let name = known.iter().find(|(_, p)| same(p, here)).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut list = vec![(name, here.to_path_buf())];
    list.extend(known.into_iter().filter(|(_, p)| !same(p, here)));
    list.into_iter()
        .map(|(name, root)| {
            Ok(Src { store: Store::new_except(&root, &[])?, config: Config::store(&root).unwrap_or_default(), name, root })
        })
        .collect()
}

/// a line of the main list
#[derive(Clone)]
struct Row {
    /// section heading (not selectable) when None
    store: Option<usize>,
    id: Option<String>,
    /// form module <cr> opens
    module: Option<&'static str>,
    line: Line<'static>,
}

impl Row {
    fn section(text: &str, note: String) -> Row {
        let t = theme::t();
        Row {
            store: None,
            id: None,
            module: None,
            line: Line::from(vec![
                Span::styled(text.to_string(), Style::default().fg(t.heading).add_modifier(Modifier::BOLD)),
                Span::styled(if note.is_empty() { note } else { format!("  {}", note) }, dim()),
            ]),
        }
    }
}

fn dim() -> Style {
    Style::default().fg(theme::t().dim)
}

fn boxed(title: String) -> Block<'static> {
    Block::bordered().border_type(BorderType::Rounded).border_style(dim()).padding(Padding::horizontal(1)).title(title)
}

fn hm(s: i64) -> String {
    format!("{}:{:02}", s / 3600, s / 60 % 60)
}

/// the store prefix in the `all` tab
fn prefix(srcs: &[Src], n: usize, all: bool) -> Vec<Span<'static>> {
    if all {
        vec![Span::styled(format!("{:<10} ", cut(&srcs[n].name, 10)), dim())]
    } else {
        vec![]
    }
}

fn cut(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + style::g().ellipsis
    }
}

fn item_row(user: &str, srcs: &[Src], n: usize, all: bool, mark: &str, i: &Item, after: String, module: Option<&'static str>) -> Row {
    let mut spans = vec![Span::raw("  ")];
    spans.extend(prefix(srcs, n, all));
    spans.push(Span::raw(format!("{} {}", mark, i.title())));
    if !after.is_empty() {
        spans.push(Span::styled(format!("  {}", after), dim()));
    }
    // who, prio, estimate, dates: as in the form's tree
    if let Some(v) = srcs[n].store.item_json(i.id()) {
        let f = crate::form::fields(&v, user, 0, false, false);
        if !f.is_empty() {
            spans.push(Span::raw("  "));
            spans.extend(f);
        }
    }
    Row { store: Some(n), id: Some(i.id().clone()), module, line: Line::from(spans) }
}

fn info_row(srcs: &[Src], n: usize, all: bool, text: String, module: Option<&'static str>) -> Row {
    let mut spans = vec![Span::raw("  ")];
    spans.extend(prefix(srcs, n, all));
    spans.push(Span::raw(text));
    Row { store: Some(n), id: None, module, line: Line::from(spans) }
}

/// the rows of tab `tab` (None: all stores)
fn rows(srcs: &[Src], tab: Option<usize>, user: &str) -> Vec<Row> {
    let g = style::g();
    let now = util::timestamp();
    let all = tab.is_none();
    let which: Vec<usize> = tab.map_or((0..srcs.len()).collect(), |t| vec![t]);
    let today = util::format_timestamp(now)[..10].to_string();
    let soon = util::format_timestamp(now + 7 * 86400)[..10].to_string();
    let mine = |i: &Item| i.assignee().is_some_and(|a| a == user);

    let mut running = vec![];
    let mut focus = vec![];
    let mut queue = vec![];
    let mut queue_more = 0;
    let mut stops = vec![];
    for &n in &which {
        let s = &srcs[n].store;
        let items: Vec<&Item> = s.get().values().collect();
        // running: timers
        let mut run: Vec<&Item> = items.iter().copied().filter(|i| i.is_started()).collect();
        run.sort_by_key(|i| (std::cmp::Reverse(i.running_seconds().unwrap_or(0)), i.id().clone()));
        for i in run {
            running.push(item_row(user, srcs, n, all, g.doing, i, hm(i.running_seconds().unwrap_or(0)), Some("mine")));
        }
        // focus: what unblocks agents first
        let blocking: Vec<&Item> =
            items.iter().copied().filter(|i| i.is_open() && mine(i) && items.iter().any(|o| o.is_open() && o.blocked_by().contains(i.id()))).collect();
        if !blocking.is_empty() {
            focus.push(info_row(srcs, n, all, format!("? {} answer(s) block agent work", blocking.len()), Some("answer")));
        }
        let review: Vec<&Item> = items.iter().copied().filter(|i| i.status() == Some(Status::Review)).collect();
        if !review.is_empty() {
            let full = reviews_full(s, &srcs[n].config).is_some();
            let text = format!("{} {} review(s){}", g.review, review.len(), if full { ", limit hit: agents stopped" } else { "" });
            focus.push(info_row(srcs, n, all, text, Some("review")));
        }
        let talk = items.iter().filter(|i| i.is_open() && mine(i) && i.tags().iter().any(|t| t == "discuss")).count();
        if talk > 0 {
            focus.push(info_row(srcs, n, all, format!("# {} to discuss (bring up in a chat)", talk), Some("mine")));
        }
        let inbox = items.iter().filter(|i| i.status() == Some(Status::Inbox)).count();
        if inbox > 0 {
            focus.push(info_row(srcs, n, all, format!("{} {} in the inbox", g.inbox, inbox), Some("inbox")));
        }
        let mut dated: Vec<(&Item, String)> = items
            .iter()
            .copied()
            .filter(|i| i.is_open())
            .filter_map(|i| i.deadline().or(i.due()).map(|d| (i, d[..10.min(d.len())].to_string())))
            .filter(|(_, d)| *d <= soon)
            .collect();
        dated.sort_by(|a, b| a.1.cmp(&b.1));
        for (i, d) in dated {
            let what = if i.deadline().is_some() { "deadline" } else { "due" };
            let late = if d < today { " (late)" } else { "" };
            focus.push(item_row(user, srcs, n, all, "!", i, format!("{} {}{}", what, d, late), Some("mine")));
        }
        // goals whose critical path misses the deadline
        for (g, deadline) in crate::gantt::goals_with_deadline(s) {
            let plan = crate::gantt::plan(s, &g, user, now);
            let finish = plan.iter().filter(|p| p.on_path).map(|p| p.end).max().unwrap_or(now);
            if finish > deadline {
                if let Some(i) = s.get_item(&g) {
                    focus.push(item_row(user, srcs, n, all, "!", i, format!("critical path {} over the deadline (g, v v v)", hm(finish - deadline)), None));
                }
            }
        }
        let mut own: Vec<&Item> = items.iter().copied().filter(|i| mine(i) && s.is_ready(i.id())).collect();
        own.sort_by_key(|i| (std::cmp::Reverse(s.score(i.id(), now)), i.id().clone()));
        for i in own.into_iter().take(OWN) {
            focus.push(item_row(user, srcs, n, all, g.todo, i, String::new(), Some("mine")));
        }
        // ai queue: ready tasks of others by score
        let mut ai: Vec<&Item> = items.iter().copied().filter(|i| !mine(i) && i.assignee().is_some() && s.is_ready(i.id()) && !i.is_started()).collect();
        ai.sort_by_key(|i| (std::cmp::Reverse(s.score(i.id(), now)), i.id().clone()));
        queue_more += ai.len().saturating_sub(QUEUE);
        for i in ai.into_iter().take(QUEUE) {
            queue.push(item_row(user, srcs, n, all, g.todo, i, String::new(), None));
        }
        if let Some(msg) = reviews_full(s, &srcs[n].config) {
            stops.push(format!("{}: {}", srcs[n].name, msg.split(';').next().unwrap_or("")));
        }
    }
    let mut out = vec![Row::section("running", String::new())];
    if running.is_empty() {
        out.push(Row { store: None, id: None, module: None, line: Line::from(Span::styled("  nothing runs", dim())) });
    }
    out.extend(running);
    out.push(Row::section("your focus", "unblocks agents first".into()));
    if focus.is_empty() {
        out.push(Row { store: None, id: None, module: None, line: Line::from(Span::styled("  nothing waits for you", dim())) });
    }
    out.extend(focus);
    let note = if queue_more > 0 { format!("+{} more", queue_more) } else { String::new() };
    out.push(Row::section("ai queue", note));
    for s in stops {
        out.push(Row { store: None, id: None, module: None, line: Line::from(Span::styled(format!("  stopped · {}", s), Style::default().fg(theme::t().error))) });
    }
    if queue.is_empty() {
        out.push(Row { store: None, id: None, module: None, line: Line::from(Span::styled("  nothing ready for agents", dim())) });
    }
    out.extend(queue);
    out
}

/// today across all stores: "5h20 · 1.2M tok · 0.4 kg CO2e"
/// this work day across all stores (from the first activity after a
/// pause of 5 h): "since 08:10 · 5:20 you · 1:10 agents · 1.2M tok · 40 g"
fn totals(srcs: &[Src], user: &str) -> String {
    let now = util::timestamp();
    let logs: Vec<Vec<crate::log::Line>> = srcs.iter().map(|s| crate::log::read(&s.root, now - 3 * 86400, now)).collect();
    let spans: Vec<(i64, i64)> = srcs.iter().zip(&logs).flat_map(|(s, l)| stats::activity(&s.store, l)).collect();
    let from = stats::workday_start(spans, now);
    if from >= now {
        return "a new day: nothing yet".into();
    }
    let mut n = stats::Numbers::default();
    for (s, l) in srcs.iter().zip(&logs) {
        n.add(&stats::window(&s.store, l, user, from, now));
    }
    let co2 = if n.co2_g >= 1000.0 { format!("{:.1} kg", n.co2_g / 1000.0) } else { format!("{:.0} g", n.co2_g) };
    let since = chrono::TimeZone::timestamp_opt(&chrono::Local, from, 0).single().map_or(String::new(), |t| t.format("%H:%M").to_string());
    format!("since {} · {} you · {} agents · {} tok · {} CO2e", since, hm(n.you), hm(n.agents), crate::footprint::human(n.tokens), co2)
}

/// the card of the selected row: title, a meta line, the body
fn card(srcs: &[Src], row: &Row, width: u16) -> (String, Vec<Line<'static>>) {
    let t = theme::t();
    let (Some(n), Some(id)) = (row.store, row.id.as_ref()) else {
        let hint = match row.module {
            Some(m) => format!("<cr> opens the form at {} in {}", m, row.store.map_or("", |n| srcs[n].name.as_str())),
            None => String::new(),
        };
        return (String::new(), vec![Line::from(Span::styled(hint, dim()))]);
    };
    let Some(i) = srcs[n].store.get_item(id) else { return (String::new(), vec![]) };
    let mut meta = vec![srcs[n].name.clone(), i.status().map_or("note".to_string(), |s| format!("{:?}", s).to_lowercase())];
    if let Some(a) = i.assignee() {
        meta.push(a.clone());
    }
    if let Some(r) = i.running_seconds() {
        meta.push(format!("running {}", hm(r)));
    } else if i.tracked_seconds() > 0 {
        meta.push(format!("tracked {}", hm(i.tracked_seconds())));
    }
    if let Some((e, guess)) = srcs[n].store.expected(id) {
        meta.push(format!("est {}{}", if guess { "~" } else { "" }, hm(e)));
    }
    if !i.commits().is_empty() {
        meta.push(format!("commits {}", i.commits().join(" ")));
    }
    let mut lines = vec![Line::from(Span::styled(meta.join(" · "), dim()))];
    if !i.tags().is_empty() {
        lines.push(Line::from(Span::styled(i.tags().iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "), Style::default().fg(t.tag))));
    }
    let body: String = i.content().lines().skip_while(|l| l.trim().is_empty()).skip(1).collect::<Vec<_>>().join("\n");
    lines.extend(markdown::render(&body, width.saturating_sub(4)));
    (i.title(), lines)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Pie,
    Stats,
    Gantt,
}

#[derive(Clone, Copy, PartialEq)]
enum Measure {
    Time,
    Tokens,
    Done,
}

impl Measure {
    fn next(self) -> Measure {
        match self {
            Measure::Time => Measure::Tokens,
            Measure::Tokens => Measure::Done,
            Measure::Done => Measure::Time,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Measure::Time => "time",
            Measure::Tokens => "tokens",
            Measure::Done => "done",
        }
    }

    fn of(self, n: &stats::Numbers) -> f64 {
        match self {
            Measure::Time => (n.you + n.agents) as f64,
            Measure::Tokens => n.tokens as f64,
            Measure::Done => n.done as f64,
        }
    }

    fn show(self, v: f64) -> String {
        match self {
            Measure::Time => hm(v as i64),
            Measure::Tokens => crate::footprint::human(v as u64),
            Measure::Done => format!("{}", v as u64),
        }
    }
}

/// a full-screen overlay over the list and the card (the tabs stay)
struct Overlay {
    kind: Kind,
    period: Period,
    measure: Measure,
    /// pie: the store whose goals are shown, None: stores
    zoom: Option<usize>,
    /// pie: the selected slice
    sel: usize,
    /// the store of the tab the overlay opened on (None: the all tab)
    only: Option<usize>,
    /// per store, for `period`
    stats: Vec<stats::StoreStats>,
    /// gantt: 0 lanes per session, 1 per store, 2 per item, 3 critical paths
    view: u8,
    /// gantt view 2: (store, goal, deadline, plan) of goals with deadlines
    plans: Vec<(usize, String, i64, Vec<crate::gantt::Planned>)>,
    /// gantt: lanes and their time span
    lanes: Vec<crate::gantt::Lane>,
    span: (i64, i64),
}

impl Overlay {
    /// on a store's tab the overlay shows that store (its goals), on the all
    /// tab every store
    fn new(kind: Kind, srcs: &[Src], user: &str, only: Option<usize>) -> Overlay {
        let mut o = Overlay { kind, period: Period::Day, measure: Measure::Time, zoom: only, sel: 0, only, stats: vec![], view: 0, plans: vec![], lanes: vec![], span: (0, 0) };
        o.refresh(srcs, user);
        o
    }

    fn refresh(&mut self, srcs: &[Src], user: &str) {
        let now = util::timestamp();
        let (from, _) = self.period.range(now);
        // the gantt looks back: the last 24 h or 7 days (today alone is
        // empty right after midnight)
        let gfrom = match self.period {
            Period::Day => now - 86400,
            Period::Week => now - 7 * 86400,
            _ => from,
        };
        let logs: Vec<Vec<crate::log::Line>> = srcs.iter().map(|s| crate::log::read(&s.root, from.min(gfrom), now)).collect();
        self.stats = srcs.iter().zip(&logs).map(|(s, l)| stats::of_store(&s.store, l, user, self.period, now)).collect();
        // the gantt: the tab's store or all, from the first hour with work
        let picked: Vec<usize> = self.only.map_or((0..srcs.len()).collect(), |n| vec![n]);
        let gsrcs: Vec<crate::gantt::Src> = picked.iter().map(|n| crate::gantt::Src { name: &srcs[*n].name, store: &srcs[*n].store, log: &logs[*n] }).collect();
        self.plans = picked
            .iter()
            .flat_map(|n| crate::gantt::goals_with_deadline(&srcs[*n].store).into_iter().map(move |(g, d)| (*n, g, d)))
            .map(|(n, g, d)| {
                let p = crate::gantt::plan(&srcs[n].store, &g, user, now);
                (n, g, d, p)
            })
            .collect();
        let mut lanes = match self.view {
            0 => crate::gantt::sessions(&gsrcs, user, gfrom, now),
            1 => crate::gantt::stores(&gsrcs, user, gfrom, now),
            _ => crate::gantt::items(&gsrcs, user, gfrom, now),
        };
        // store indices of the lanes refer to `picked`; map back for colors
        for l in lanes.iter_mut() {
            for b in l.bars.iter_mut() {
                b.store = picked[b.store];
            }
        }
        let first = lanes.iter().flat_map(|l| &l.bars).map(|b| b.start).min().unwrap_or(gfrom).max(gfrom);
        let start = if self.period == Period::Day { first - first.rem_euclid(3600) } else { gfrom };
        self.span = (start.max(gfrom), now);
        self.lanes = lanes;
    }

    /// (name, value) of the pie's slices, largest first
    fn slices(&self, srcs: &[Src]) -> Vec<(String, f64)> {
        let mut v: Vec<(String, f64)> = match self.zoom {
            None => srcs.iter().zip(&self.stats).map(|(s, st)| (s.name.clone(), self.measure.of(&st.numbers))).collect(),
            Some(n) => self.stats[n].goals.iter().map(|g| (if g.id.is_empty() { "other".into() } else { g.id.clone() }, self.measure.of(&g.numbers))).collect(),
        };
        v.retain(|(_, x)| *x > 0.0);
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v
    }
}

/// what the main screen shows beside the lists on wider terminals,
/// for the open tab
struct Side {
    /// the pie of this week
    pie: Overlay,
    /// who-gantt of the last 24 h
    who: Overlay,
    /// one gantt line per store, last 24 h
    stores: Overlay,
    /// the stats line: the last 7 days per day, their numbers, and the
    /// store numbers (review, overdue, streak)
    days: Vec<i64>,
    last7: stats::Numbers,
    now: Vec<stats::StoreStats>,
    only: Option<usize>,
}

impl Side {
    fn new(only: Option<usize>, srcs: &[Src], user: &str) -> Side {
        let mut pie = Overlay::new(Kind::Pie, srcs, user, only);
        pie.period = Period::Days7;
        let mut stores = Overlay::new(Kind::Gantt, srcs, user, only);
        stores.view = 1;
        let mut side = Side { pie, who: Overlay::new(Kind::Gantt, srcs, user, only), stores, days: vec![], last7: Default::default(), now: vec![], only };
        side.refresh(srcs, user);
        side
    }

    fn retab(&mut self, only: Option<usize>, srcs: &[Src], user: &str) {
        *self = Side::new(only, srcs, user);
    }

    fn refresh(&mut self, srcs: &[Src], user: &str) {
        for o in [&mut self.pie, &mut self.who, &mut self.stores] {
            o.refresh(srcs, user);
        }
        let now = util::timestamp();
        let picked: Vec<usize> = self.only.map_or((0..srcs.len()).collect(), |n| vec![n]);
        self.days = vec![0; 7];
        self.last7 = Default::default();
        for n in &picked {
            let log = crate::log::read(&srcs[*n].root, now - 8 * 86400, now);
            for (k, d) in stats::days(&srcs[*n].store, &log, user, 7, now).into_iter().enumerate() {
                self.days[k] += d;
            }
            self.last7.add(&stats::window(&srcs[*n].store, &log, user, now - 7 * 86400, now));
        }
        self.now = picked.iter().map(|n| self.pie.stats[*n].clone()).collect();
    }

    /// "7 days ▂▅█▃▁▆█ 14:20 · done 86 · streak 3 d · review 6 · overdue 0"
    fn line(&self) -> Line<'static> {
        let t = theme::t();
        let review: usize = self.now.iter().map(|s| s.review).sum();
        let overdue: usize = self.now.iter().map(|s| s.overdue).sum();
        let streak = self.now.iter().map(|s| s.streak).max().unwrap_or(0);
        let n = &self.last7;
        Line::from(vec![
            Span::styled(" 7 days ", dim()),
            Span::styled(crate::commands::spark(&self.days), Style::default().fg(t.hint)),
            Span::styled(
                format!("  you {} · agents {} · done {} · streak {} d · review {} · ", hm(n.you), hm(n.agents), n.done, streak, review),
                dim(),
            ),
            Span::styled(format!("overdue {}", overdue), if overdue > 0 { Style::default().fg(t.error) } else { dim() }),
        ])
    }
}

struct View {
    tab: usize,
    sel: usize,
    note: Option<String>,
    overlay: Option<Overlay>,
}

/// slice colors, from the theme
fn palette() -> Vec<ratatui::style::Color> {
    let t = theme::t();
    vec![t.hint, t.mine, t.ok, t.code, t.bold, t.tag, t.error, t.heading]
}

fn paint_pie(f: &mut ratatui::Frame, area: Rect, srcs: &[Src], o: &Overlay) {
    use ratatui::widgets::canvas::{Canvas, Points};
    let slices = o.slices(srcs);
    let total: f64 = slices.iter().map(|(_, v)| v).sum();
    let what = match o.zoom {
        None => "stores".to_string(),
        Some(n) => format!("goals in {}", srcs[n].name),
    };
    let block = boxed(format!(" {} · {} · {} ", o.period.name(), o.measure.name(), what));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if slices.is_empty() {
        f.render_widget(Paragraph::new(Span::styled("nothing in this period yet (d: longer period, m: other measure)", dim())), inner);
        return;
    }
    // half blocks: two square pixels per cell, each its own color; a
    // circle when the pie is twice as wide as high
    let h = inner.height.min(inner.width / 4).clamp(4, 10);
    let top = inner.y + 1.min(inner.height.saturating_sub(h));
    let pie = Rect { x: inner.x + 2, y: top, width: h * 2, height: h };
    let legend = Rect { x: inner.x + h * 2 + 6, y: top, width: inner.width.saturating_sub(h * 2 + 6), height: inner.height.saturating_sub(top - inner.y) };
    let colors = palette();
    // each slice from its start angle, clockwise from 12 o'clock
    let mut bounds = vec![];
    let mut acc = 0.0;
    for (_, v) in &slices {
        bounds.push(acc);
        acc += v / total;
    }
    let (w, ht) = (pie.width as usize, pie.height as usize * 2);
    let mut points: Vec<Vec<(f64, f64)>> = vec![vec![]; slices.len()];
    for px in 0..w {
        for py in 0..ht {
            let x = (px as f64 + 0.5) / w as f64 * 2.0 - 1.0;
            let y = (py as f64 + 0.5) / ht as f64 * 2.0 - 1.0;
            if x * x + y * y > 1.0 {
                continue;
            }
            let turn = (x.atan2(y) / std::f64::consts::TAU).rem_euclid(1.0);
            let n = bounds.iter().rposition(|b| *b <= turn).unwrap_or(0);
            points[n].push((x, y));
        }
    }
    let canvas = Canvas::default().marker(ratatui::symbols::Marker::HalfBlock).x_bounds([-1.0, 1.0]).y_bounds([-1.0, 1.0]).paint(|ctx| {
        for (n, p) in points.iter().enumerate() {
            ctx.draw(&Points { coords: p, color: colors[n % colors.len()] });
        }
    });
    f.render_widget(canvas, pie);
    // names as wide as the legend allows
    let nw = (legend.width as usize).saturating_sub(4 + 7 + 5).clamp(6, 16);
    let mut lines: Vec<Line> = slices
        .iter()
        .enumerate()
        .map(|(n, (name, v))| {
            let here = n == o.sel;
            let style = if here { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() };
            Line::from(vec![
                Span::raw(if here { "› " } else { "  " }),
                Span::styled("■ ", Style::default().fg(colors[n % colors.len()])),
                Span::styled(format!("{:<nw$} {:>6} {:>3.0}%", cut(name, nw), o.measure.show(*v), v / total * 100.0, nw = nw), style),
            ])
        })
        .collect();
    let n = o.zoom.map_or_else(|| stats::sum(&o.stats), |z| o.stats[z].numbers.clone());
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(format!("you {} · agents {}", hm(n.you), hm(n.agents)), dim())));
    lines.push(Line::from(Span::styled(format!("{:.0} Wh · {:.0} g CO2e", n.wh, n.co2_g), dim())));
    lines.push(Line::from(Span::styled(format!("{:.1} l water", n.water_l), dim())));
    f.render_widget(Paragraph::new(lines), legend);
}

/// gantt view 2: a goal's open tasks from now to its deadline, the
/// critical path in the `mine` color
fn paint_path(f: &mut ratatui::Frame, area: Rect, srcs: &[Src], o: &Overlay) {
    let t = theme::t();
    let scope = o.only.map_or("all stores".to_string(), |n| srcs[n].name.clone());
    let block = boxed(format!(" gantt · critical path · {} ", scope));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let Some((n, goal, deadline, plan)) = o.plans.get(o.sel.min(o.plans.len().saturating_sub(1))) else {
        f.render_widget(Paragraph::new(Span::styled("no goal with a deadline here (cake edit <goal> --deadline 30.11.)", dim())), inner);
        return;
    };
    let now = util::timestamp();
    let title = srcs[*n].store.get_item(goal).map_or(goal.clone(), |i| i.title());
    let left = deadline - now;
    let end = plan.iter().map(|p| p.end).max().unwrap_or(now).max(*deadline).max(now + 3600);
    let name_w = plan.iter().map(|p| p.id.chars().count()).max().unwrap_or(4).clamp(4, 22) + 6;
    let width = (inner.width as usize).saturating_sub(name_w + 2).max(10);
    let col = |x: i64| (((x - now) as f64 / (end - now) as f64) * width as f64) as usize;
    let dl = col(*deadline).min(width - 1);
    let mut lines = vec![Line::from(vec![
        Span::styled(format!("{} ", title), Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(format!("· deadline {} · {} left · goal {}/{} (j/k)", util::format_timestamp(*deadline)[..10].to_string(), if left > 86400 { format!("{} d", left / 86400) } else { hm(left.max(0)) }, o.sel.min(o.plans.len() - 1) + 1, o.plans.len()), dim()),
    ])];
    let mut axis = vec![' '; width];
    axis[0] = '┊';
    axis[dl] = '│';
    lines.push(Line::from(Span::styled(format!("{:w$}  now{}", "", axis.iter().skip(3).collect::<String>(), w = name_w), dim())));
    for p in plan {
        let len = if p.question { "◆".to_string() } else { format!("{}{}", if p.guess { "~" } else { "" }, hm(p.end - p.start)) };
        let mut cells: Vec<Span> = vec![Span::styled(format!("{:w$}  ", format!("{} {}", cut(&p.id, name_w - 6), len), w = name_w), dim())];
        let (a, b) = (col(p.start), col(p.end).max(col(p.start) + 1).min(width));
        let color = if p.question { t.question } else if p.on_path { t.mine } else { t.hint };
        for c in 0..width {
            let ch = if p.question && c == a { "◆" } else if c >= a && c < b && !p.question { "█" } else if c == dl { "│" } else { " " };
            cells.push(if ch == "│" { Span::styled(ch, dim()) } else { Span::styled(ch, Style::default().fg(color)) });
        }
        lines.push(Line::from(cells));
    }
    lines.push(Line::default());
    let path: Vec<&crate::gantt::Planned> = plan.iter().filter(|p| p.on_path).collect();
    let total: i64 = path.iter().map(|p| p.end - p.start).sum();
    let finish = path.iter().map(|p| p.end).max().unwrap_or(now);
    let names: Vec<&str> = path.iter().map(|p| p.id.as_str()).collect();
    let (verdict, style) = if finish <= *deadline { ("fits ✓".to_string(), Style::default().fg(t.ok)) } else { (format!("{} over", hm(finish - deadline)), Style::default().fg(t.error)) };
    lines.push(Line::from(vec![Span::styled(format!("critical path: {} = {} · ", names.join(" → "), hm(total)), dim()), Span::styled(verdict, style)]));
    lines.push(Line::from(Span::styled("estimates from --est, ~ = history median or 1 h; questions ◆ hold the chain", dim())));
    f.render_widget(Paragraph::new(lines), inner);
}

fn paint_gantt(f: &mut ratatui::Frame, area: Rect, srcs: &[Src], o: &Overlay) {
    use crate::gantt;
    if o.view == 3 {
        return paint_path(f, area, srcs, o);
    }
    let (from, to) = o.span;
    let scope = o.only.map_or("all stores".to_string(), |n| srcs[n].name.clone());
    let block = boxed(format!(" gantt · {} · {} · {} ", if o.period == Period::Day { "24 h" } else { "7 days" }, ["who", "stores", "items"][o.view.min(2) as usize], scope));
    // items: a cursor row and its card below
    let (area, card_area) = if o.view == 2 && !o.lanes.is_empty() {
        let [a, c] = Layout::vertical([Constraint::Fill(1), Constraint::Percentage(35)]).areas(area);
        (a, Some(c))
    } else {
        (area, None)
    };
    let inner = block.inner(area);
    f.render_widget(block, area);
    let name_w = o.lanes.iter().map(|l| l.name.chars().count()).max().unwrap_or(4).clamp(4, 22);
    let width = (inner.width as usize).saturating_sub(name_w + 2).max(10);
    let colors = palette();
    let mut lines = vec![Line::from(Span::styled(format!("{:w$}  {}", "", gantt::axis(from, to, width), w = name_w), dim()))];
    let visible: Vec<&gantt::Lane> = o.lanes.iter().filter(|l| !l.bars.is_empty()).collect();
    if visible.is_empty() {
        lines.push(Line::from(Span::styled("nothing ran in this period (d: week)", dim())));
    }
    // rows in view: the cursor row stays visible (items scroll)
    let room = (inner.height as usize).saturating_sub(5).max(1);
    let top = if o.view == 2 { (o.sel + 1).saturating_sub(room) } else { 0 };
    for (k, l) in visible.iter().enumerate().skip(top).take(room) {
        let here = o.view == 2 && k == o.sel;
        let name_style = if here { Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED) } else { dim() };
        let mut spans = vec![Span::styled(format!("{:w$}", cut(&l.name, name_w), w = name_w), name_style), Span::raw("  ")];
        spans.extend(gantt::render(l, from, to, width, &colors, o.view == 1, style::ascii()).spans);
        lines.push(Line::from(spans));
    }
    if visible.len() > top + room {
        lines.push(Line::from(Span::styled(format!("{:w$}  … {} more (j/k)", "", visible.len() - top - room, w = name_w), dim())));
    }
    lines.push(Line::default());
    // the stores' colors, then what the marks mean
    let mut legend: Vec<Span> = vec![];
    let shown: Vec<usize> = o.only.map_or((0..srcs.len()).collect(), |n| vec![n]);
    for n in shown {
        legend.push(Span::styled("■ ", Style::default().fg(colors[n % colors.len()])));
        legend.push(Span::styled(format!("{}  ", srcs[n].name), dim()));
    }
    let marks = match o.view {
        2 => "█ timer  ▒ your chat on it",
        0 => "you: █ chat ▒ in cake · agents: ░ 1 ▒ 2 ▓ 3 █ 4+ at once · waits on you",
        _ => "░ 1 at once  ▒ 2  ▓ 3  █ 4+  ⁚ waits on you",
    };
    legend.push(Span::styled(marks, dim()));
    lines.push(Line::from(legend));
    let (max, avg, waited) = gantt::summary(&o.lanes, from, to);
    lines.push(Line::from(Span::styled(format!("at once: max {} · avg {:.1} · agents waited {} on you", max, avg, hm(waited)), dim())));
    f.render_widget(Paragraph::new(lines), inner);
    if let (Some(c), Some(l)) = (card_area, o.lanes.get(o.sel)) {
        let row = Row { store: l.bars.first().map(|b| b.store), id: Some(l.name.clone()), module: None, line: Line::default() };
        let (title, body) = card(srcs, &row, c.width);
        f.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }).block(boxed(format!(" {} ", title))), c);
    }
}

/// the store lanes of the last 24 h, compact: axis and one line per store
fn paint_strip(f: &mut ratatui::Frame, area: Rect, srcs: &[Src], o: &Overlay) {
    let (from, to) = o.span;
    let block = boxed(" last 24 h ".into());
    let inner = block.inner(area);
    f.render_widget(block, area);
    let name_w = o.lanes.iter().map(|l| l.name.chars().count()).max().unwrap_or(4).clamp(4, 16);
    let width = (inner.width as usize).saturating_sub(name_w + 2).max(10);
    let mut lines = vec![Line::from(Span::styled(format!("{:w$}  {}", "", crate::gantt::axis(from, to, width), w = name_w), dim()))];
    let colors = palette();
    let mut any = false;
    for l in o.lanes.iter().filter(|l| !l.bars.is_empty()) {
        any = true;
        let mut spans = vec![Span::styled(format!("{:w$}  ", cut(&l.name, name_w), w = name_w), dim())];
        spans.extend(crate::gantt::render(l, from, to, width, &colors, true, style::ascii()).spans);
        lines.push(Line::from(spans));
    }
    if !any {
        lines.push(Line::from(Span::styled("nothing ran in the last 24 h", dim())));
    }
    let _ = srcs;
    f.render_widget(Paragraph::new(lines), inner);
}

fn paint_stats(f: &mut ratatui::Frame, area: Rect, srcs: &[Src], o: &Overlay) {
    let t = theme::t();
    let block = boxed(format!(" stats · {} ", o.period.name()));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let w = srcs.iter().map(|s| s.name.chars().count()).max().unwrap_or(5).clamp(5, 16);
    // narrow (the dashboard's side column): no kWh, no sparkline
    let compact = inner.width < 80;
    let head = if compact {
        format!("{:w$}  {:>6}  {:>6}  {:>7}  {:>7}  {:>4}", "", "you", "agents", "tokens", "CO2e", "done", w = w)
    } else {
        format!("{:w$}  {:>6}  {:>6}  {:>7}  {:>6}  {:>8}  {:>4}  {}", "", "you", "agents", "tokens", "kWh", "CO2e", "done", o.period.bucket(), w = w)
    };
    let row = |name: &str, n: &stats::Numbers, buckets: String| {
        if compact {
            return format!("{:w$}  {:>6}  {:>6}  {:>7}  {:>5.0} g  {:>4}", cut(name, w), hm(n.you), hm(n.agents), crate::footprint::human(n.tokens), n.co2_g, n.done, w = w);
        }
        format!(
            "{:w$}  {:>6}  {:>6}  {:>7}  {:>6.2}  {:>6.0} g  {:>4}  {}",
            cut(name, w), hm(n.you), hm(n.agents), crate::footprint::human(n.tokens), n.wh / 1000.0, n.co2_g, n.done, buckets, w = w
        )
    };
    let mut lines = vec![Line::from(Span::styled(head, dim()))];
    let shown: Vec<usize> = o.only.map_or((0..srcs.len()).collect(), |n| vec![n]);
    for &n in &shown {
        let st = &o.stats[n];
        lines.push(Line::from(row(&srcs[n].name, &st.numbers, crate::commands::spark(&st.buckets.iter().map(|(_, v)| *v).collect::<Vec<_>>()))));
    }
    if shown.len() > 1 {
        lines.push(Line::from(Span::styled(row("all", &stats::sum(&o.stats), String::new()), Style::default().add_modifier(Modifier::BOLD))));
    }
    // a store's tab: its goals below
    if let Some(n) = o.only {
        for g in &o.stats[n].goals {
            let name = format!("  {}", if g.id.is_empty() { "other" } else { &g.id });
            lines.push(Line::from(Span::styled(row(&name, &g.numbers, String::new()), dim())));
        }
    }
    lines.push(Line::default());
    let picked: Vec<&stats::StoreStats> = shown.iter().map(|n| &o.stats[*n]).collect();
    let (overdue, review, questions) = picked.iter().fold((0, 0, 0), |a, s| (a.0 + s.overdue, a.1 + s.review, a.2 + s.questions));
    let streak = picked.iter().map(|s| s.streak).max().unwrap_or(0);
    let late = Style::default().fg(if overdue > 0 { t.error } else { t.dim });
    lines.push(Line::from(vec![
        Span::styled(format!("overdue {}", overdue), late),
        Span::styled(format!(" · review {} · questions {} · streak {} d", review, questions, streak), dim()),
    ]));
    f.render_widget(Paragraph::new(lines), inner);
}

/// terminal widths where the main screen shows more
const MEDIUM: u16 = 100;
const WIDE: u16 = 120;

fn paint(f: &mut ratatui::Frame, srcs: &[Src], v: &View, rows: &[Row], total: &str, side: &Side, user: &str) {
    let t = theme::t();
    let area = f.area();
    let col = Rect { x: area.x + 1, width: area.width.saturating_sub(2), ..area };
    let wide = col.width >= WIDE && v.overlay.is_none();
    let medium = col.width >= MEDIUM && v.overlay.is_none();
    // medium: a stats line and today's store lanes above the card
    let strip_h = if medium && !wide { side.stores.lanes.iter().filter(|l| !l.bars.is_empty()).count().max(1) as u16 + 3 } else { 0 };
    let [head, statline, list, strip, card_area, keys] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(u16::from(medium)),
        Constraint::Fill(1),
        Constraint::Length(strip_h),
        Constraint::Percentage(if wide { 30 } else { 40 }),
        Constraint::Length(1),
    ])
    .areas(col);
    if medium {
        f.render_widget(Paragraph::new(side.line()), statline);
    }
    if strip_h > 0 {
        paint_strip(f, strip, srcs, &side.stores);
    }
    // wide: pie and who-gantt to the right of the lists
    let list = if wide {
        let [l, r] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(col.width * 45 / 100)]).areas(list);
        // pie, the who-gantt as tall as its lanes, the stats per goal below
        let lanes = side.who.lanes.iter().filter(|l| !l.bars.is_empty()).count() as u16;
        let [pie, gantt, table] = Layout::vertical([Constraint::Length(12), Constraint::Length(lanes.max(1) + 7), Constraint::Fill(1)]).areas(r);
        paint_pie(f, pie, srcs, &side.pie);
        paint_gantt(f, gantt, srcs, &side.who);
        if table.height >= 5 {
            paint_stats(f, table, srcs, &side.pie);
        }
        l
    } else {
        list
    };
    // tabs, totals on the right
    let mut tabs: Vec<Span> = vec![];
    for (n, name) in std::iter::once("all").chain(srcs.iter().map(|s| s.name.as_str())).enumerate() {
        if n > 0 {
            tabs.push(Span::styled(" · ", dim()));
        }
        tabs.push(if n == v.tab { Span::styled(name.to_uppercase(), Style::default().fg(t.bold).add_modifier(Modifier::BOLD)) } else { Span::raw(name.to_string()) });
    }
    let used: usize = tabs.iter().map(|s| s.width()).sum();
    let room = (head.width as usize).saturating_sub(5 + used);
    tabs.push(Span::styled(format!("{:>w$}", cut(total, room), w = room), dim()));
    // who and when, in the top border
    let clock = chrono::Local::now().format("%a %H:%M").to_string();
    let block = boxed(String::new()).title(Line::from(Span::styled(format!(" {} · {} ", user, clock), dim())).right_aligned());
    f.render_widget(Paragraph::new(Line::from(tabs)).block(block), head);
    if let Some(o) = &v.overlay {
        let body = Rect { height: list.height + card_area.height, ..list };
        match o.kind {
            Kind::Pie => paint_pie(f, body, srcs, o),
            Kind::Stats => paint_stats(f, body, srcs, o),
            Kind::Gantt => paint_gantt(f, body, srcs, o),
        }
        let k = match o.kind {
            Kind::Pie => "m measure  d period  j/k slice  <cr> goals  <bs> stores  s stats  g gantt  q back",
            Kind::Stats => "d period  p pie  g gantt  q back",
            Kind::Gantt => "v who/stores/items/path  d day/week  j/k row  <cr> tree  e edit  p pie  s stats  q back",
        };
        f.render_widget(Paragraph::new(Line::from(Span::styled(k, dim()))), keys);
        return;
    }
    // the list, the cursor row kept in view
    let h = list.height as usize;
    let top = (v.sel + 2).saturating_sub(h);
    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .skip(top)
        .take(h)
        .map(|(n, r)| {
            let mut l = r.line.clone();
            if n == v.sel {
                l.spans.insert(0, Span::styled("›", Style::default().fg(t.bold).add_modifier(Modifier::BOLD)));
                for s in l.spans.iter_mut().skip(1) {
                    s.style = s.style.add_modifier(Modifier::BOLD);
                }
            } else {
                l.spans.insert(0, Span::raw(" "));
            }
            l
        })
        .collect();
    f.render_widget(Paragraph::new(lines), list);
    // the card
    let (title, body) = rows.get(v.sel).map_or((String::new(), vec![]), |r| card(srcs, r, card_area.width));
    let block = boxed(if title.is_empty() { String::new() } else { format!(" {} ", title) });
    f.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }).block(block), card_area);
    let keys_line = match &v.note {
        Some(n) => Line::from(Span::styled(n.clone(), Style::default().fg(t.error))),
        None => Line::from(Span::styled("h/l store  j/k  <cr> open  a add  e edit  F form  T tree  D dump  p pie  s stats  g gantt  q quit", dim())),
    };
    f.render_widget(Paragraph::new(keys_line), keys);
}

fn exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cake"))
}

/// runs `cake --store <root> <args>` on the real terminal, then back
fn suspend(term: &mut ratatui::DefaultTerminal, root: &Path, args: &[&str]) -> Result<(), Box<dyn Error>> {
    ratatui::restore();
    let status = Command::new(exe()).arg("--store").arg(root).args(args).status();
    *term = ratatui::init();
    status?;
    Ok(())
}

fn selectable(rows: &[Row], from: usize, down: bool) -> usize {
    let ok = |n: usize| rows.get(n).is_some_and(|r| r.store.is_some());
    let mut n = from;
    loop {
        if down {
            if n + 1 >= rows.len() {
                return if ok(from) { from } else { first(rows) };
            }
            n += 1;
        } else {
            if n == 0 {
                return if ok(from) { from } else { first(rows) };
            }
            n -= 1;
        }
        if ok(n) {
            return n;
        }
    }
}

fn first(rows: &[Row]) -> usize {
    rows.iter().position(|r| r.store.is_some()).unwrap_or(0)
}

pub fn run(root: &Path, user: &str) -> Result<(), Box<dyn Error>> {
    let mut srcs = load(root)?;
    // tab 0 is all, the nearest store is tab 1
    let mut v = View { tab: 1.min(srcs.len()), sel: 0, note: None, overlay: None };
    let mut term = ratatui::init();
    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut total = totals(&srcs, user);
        let mut side = Side::new((v.tab > 0).then(|| v.tab - 1), &srcs, user);
        let mut last = std::time::Instant::now();
        loop {
            let tab = (v.tab > 0).then(|| v.tab - 1);
            let rows = rows(&srcs, tab, user);
            if rows.get(v.sel).map_or(true, |r| r.store.is_none()) {
                v.sel = selectable(&rows, v.sel, true);
            }
            term.draw(|f| paint(f, &srcs, &v, &rows, &total, &side, user))?;
            if !event::poll(Duration::from_millis(500))? {
                if last.elapsed() >= Duration::from_secs(2) {
                    srcs = load(root)?;
                    total = totals(&srcs, user);
                    if let Some(o) = v.overlay.as_mut() {
                        o.refresh(&srcs, user);
                    }
                    side.refresh(&srcs, user);
                    last = std::time::Instant::now();
                }
                continue;
            }
            let Event::Key(k) = event::read()? else { continue };
            if k.kind != KeyEventKind::Press {
                continue;
            }
            v.note = None;
            let tabs = srcs.len() + 1;
            // h/l change the tab also under an overlay, which follows it
            let step = match k.code {
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => Some(1),
                KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => Some(tabs - 1),
                _ => None,
            };
            if let (Some(step), true) = (step, v.overlay.is_some()) {
                v.tab = (v.tab + step) % tabs;
                let only = (v.tab > 0).then(|| v.tab - 1);
                let o = v.overlay.as_mut().unwrap();
                (o.only, o.zoom, o.sel) = (only, only, 0);
                o.refresh(&srcs, user);
                side.retab(only, &srcs, user);
                continue;
            }
            if let Some(o) = v.overlay.as_mut() {
                let count = o.slices(&srcs).len();
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => v.overlay = None,
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                    KeyCode::Char('d') => {
                        // the gantt has a day and a week
                        o.period = match (o.kind, o.period) {
                            (Kind::Gantt, Period::Day) => Period::Week,
                            (Kind::Gantt, _) => Period::Day,
                            (_, p) => p.next(),
                        };
                        o.refresh(&srcs, user);
                    }
                    KeyCode::Char('v') if o.kind == Kind::Gantt => {
                        o.view = (o.view + 1) % 4;
                        o.sel = 0;
                        o.refresh(&srcs, user);
                    }
                    KeyCode::Char('g') => {
                        o.kind = Kind::Gantt;
                        if !matches!(o.period, Period::Day | Period::Week) {
                            o.period = Period::Day;
                        }
                        o.refresh(&srcs, user);
                    }
                    KeyCode::Char('m') => o.measure = o.measure.next(),
                    KeyCode::Char('j') | KeyCode::Down if o.kind == Kind::Gantt => {
                        let n = if o.view == 2 { o.lanes.len() } else { o.plans.len() };
                        o.sel = (o.sel + 1).min(n.saturating_sub(1));
                    }
                    // items: open the tree on it, or edit it
                    KeyCode::Enter | KeyCode::Char('e') if o.kind == Kind::Gantt && o.view == 2 => {
                        if let Some((n, id)) = o.lanes.get(o.sel).and_then(|l| l.bars.first().map(|b| (b.store, l.name.clone()))) {
                            let args: Vec<&str> = if k.code == KeyCode::Enter { vec!["form", "all", "--at", &id] } else { vec!["edit", &id] };
                            suspend(&mut term, &srcs[n].root.clone(), &args)?;
                            srcs = load(root)?;
                            if let Some(o) = v.overlay.as_mut() {
                                o.refresh(&srcs, user);
                            }
                        }
                        continue;
                    }
                    KeyCode::Char('j') | KeyCode::Down => o.sel = (o.sel + 1).min(count.saturating_sub(1)),
                    KeyCode::Char('k') | KeyCode::Up => o.sel = o.sel.saturating_sub(1),
                    KeyCode::Enter if o.kind == Kind::Pie && o.zoom.is_none() => {
                        let name = o.slices(&srcs).get(o.sel).map(|(n, _)| n.clone());
                        o.zoom = name.and_then(|n| srcs.iter().position(|s| s.name == n));
                        o.sel = 0;
                    }
                    KeyCode::Backspace if o.only.is_none() => (o.zoom, o.sel) = (None, 0),
                    KeyCode::Char('p') => o.kind = Kind::Pie,
                    KeyCode::Char('s') => o.kind = Kind::Stats,
                    _ => (),
                }
                continue;
            }
            let tabs = srcs.len() + 1;
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => {
                    let step = if matches!(k.code, KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab) { 1 } else { tabs - 1 };
                    (v.tab, v.sel) = ((v.tab + step) % tabs, 0);
                    side.retab((v.tab > 0).then(|| v.tab - 1), &srcs, user);
                }
                KeyCode::Char('j') | KeyCode::Down => v.sel = selectable(&rows, v.sel, true),
                KeyCode::Char('k') | KeyCode::Up => v.sel = selectable(&rows, v.sel, false),
                KeyCode::Enter => match rows.get(v.sel).and_then(|r| r.store.zip(r.module)) {
                    Some((n, m)) => {
                        suspend(&mut term, &srcs[n].root.clone(), &["form", m])?;
                        srcs = load(root)?;
                        total = totals(&srcs, user);
                    }
                    None => v.note = Some("<cr> works on the focus lines; e edits an item".into()),
                },
                KeyCode::Char('e') => match rows.get(v.sel).and_then(|r| r.store.zip(r.id.clone())) {
                    Some((n, id)) => {
                        suspend(&mut term, &srcs[n].root.clone(), &["edit", &id])?;
                        srcs = load(root)?;
                    }
                    None => v.note = Some("no item on this line".into()),
                },
                KeyCode::Char('a') => {
                    // into the inbox of this tab's store (all: the nearest)
                    let n = tab.unwrap_or(0);
                    suspend(&mut term, &srcs[n].root.clone(), &["add"])?;
                    srcs = load(root)?;
                }
                KeyCode::Char('p') => v.overlay = Some(Overlay::new(Kind::Pie, &srcs, user, tab)),
                KeyCode::Char('s') => v.overlay = Some(Overlay::new(Kind::Stats, &srcs, user, tab)),
                KeyCode::Char('g') => v.overlay = Some(Overlay::new(Kind::Gantt, &srcs, user, tab)),
                // the form: what waits (F), the whole tree (T), a dump (D)
                KeyCode::Char(c @ ('F' | 'T' | 'D')) => {
                    let n = tab.unwrap_or(0);
                    let args: &[&str] = match c {
                        'F' => &["form", "news", "inbox", "answer", "review"],
                        'T' => &["form", "all"],
                        // the new ideas right after, to sort and change
                        _ => &["form", "dump", "inbox"],
                    };
                    suspend(&mut term, &srcs[n].root.clone(), args)?;
                    srcs = load(root)?;
                    total = totals(&srcs, user);
                    side.refresh(&srcs, user);
                }
                _ => (),
            }
        }
    })();
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(name: &str, items: Vec<Item>) -> Src {
        let mut store = Store::new(Path::new("/definitely/does/not/exist/cake-dash-test")).unwrap();
        for i in items {
            store.add(i).unwrap();
        }
        Src { name: name.into(), root: PathBuf::from("/nonexistent"), store, config: StoreConfig::default() }
    }

    fn task(id: &str, who: &str) -> Item {
        let mut i = Item::new(id.into(), vec![], None, vec![]);
        i.set_content(format!("# {}\n", id));
        i.set_status(Status::Task).unwrap();
        i.set_assignee(Some(who.into()));
        i
    }

    fn text(rows: &[Row]) -> Vec<String> {
        rows.iter().map(|r| r.line.to_string()).collect()
    }

    #[test]
    fn focus_puts_blocking_answers_first_and_the_queue_lists_agent_work() {
        style::init("unicode");
        let mut work = task("work", "claude-code");
        work.set_blocked_by(vec!["ask".into()]);
        let mut review = task("done-ish", "claude-code");
        review.set_status(Status::Review).unwrap();
        let s = src("cake", vec![work, task("ask", "jul"), review, task("next", "ai"), task("mine", "jul")]);
        let rows = rows(&[s], Some(0), "jul");
        let t = text(&rows);
        let at = |needle: &str| t.iter().position(|l| l.contains(needle)).unwrap_or_else(|| panic!("{} not in {:#?}", needle, t));
        assert!(at("answer(s) block") < at("1 review(s)"));
        assert!(at("1 review(s)") < at("ai queue"));
        assert!(at("ai queue") < at("○ next"), "ready agent task in the queue");
        assert!(!t.iter().any(|l| l.contains("○ work")), "blocked work waits");
        assert!(at("○ mine") < at("ai queue"), "own ready task in the focus");
        assert_eq!(rows[at("answer(s) block")].module, Some("answer"));
    }

    #[test]
    fn the_all_tab_prefixes_lines_with_the_store() {
        style::init("unicode");
        let a = src("cake", vec![task("one", "ai")]);
        let b = src("habitats", vec![task("two", "ai")]);
        let t = text(&rows(&[a, b], None, "jul"));
        assert!(t.iter().any(|l| l.contains("cake") && l.contains("one")));
        assert!(t.iter().any(|l| l.contains("habitats") && l.contains("two")));
    }

    #[test]
    fn selection_skips_headings() {
        style::init("unicode");
        let rows = rows(&[src("cake", vec![task("a", "ai"), task("b", "ai")])], Some(0), "jul");
        let a = first(&rows);
        let b = selectable(&rows, a, true);
        assert!(rows[a].store.is_some() && rows[b].store.is_some() && b > a);
        assert_eq!(selectable(&rows, b, true), b, "stays on the last");
    }

    #[test]
    fn pie_slices_are_stores_or_a_stores_goals_largest_first() {
        let mut goal = Item::new("g".into(), vec![], None, vec!["goal".into()]);
        goal.set_content("# g\n".into());
        let mut a = task("a", "ai");
        a.set_status(Status::Done).unwrap();
        let mut b = task("b", "ai");
        b.set_status(Status::Done).unwrap();
        b.set_parent(Some("g".into()));
        let srcs = vec![src("small", vec![task("x", "ai")]), src("big", vec![goal, a, b])];
        let mut o = Overlay::new(Kind::Pie, &srcs, "jul", None);
        o.measure = Measure::Done;
        assert_eq!(o.slices(&srcs), vec![("big".to_string(), 2.0)], "empty stores have no slice");
        o.zoom = Some(1);
        let mut names: Vec<String> = o.slices(&srcs).into_iter().map(|(n, _)| n).collect();
        names.sort();
        assert_eq!(names, ["g", "other"]);
    }
}
