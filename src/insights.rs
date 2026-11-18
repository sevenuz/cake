//! the stats overlay's sections beyond the overview: rhythm, flow, agents
//! and fun. Pure: stores and log lines in, lines out.

use crate::item::{Item, Status};
use crate::log::Line as LogLine;
use crate::stats;
use crate::store::Store;
use chrono::{Datelike, Local, TimeZone, Timelike};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::collections::BTreeMap;

/// one store's input
pub struct Src<'a> {
    pub store: &'a Store,
    pub log: &'a [LogLine],
}

pub struct Look {
    pub dim: Style,
    pub you: Color,
    pub agents: Color,
}

fn hm(s: i64) -> String {
    format!("{}:{:02}", s / 3600, s / 60 % 60)
}

fn days(s: i64) -> String {
    if s >= 86400 {
        format!("{:.1} d", s as f64 / 86400.0)
    } else {
        hm(s)
    }
}

fn median(mut v: Vec<i64>) -> Option<i64> {
    v.sort();
    v.get(v.len() / 2).copied().filter(|_| !v.is_empty())
}

fn local(t: i64) -> chrono::DateTime<Local> {
    Local.timestamp_opt(t, 0).single().unwrap_or_default()
}

/// the user's own task (else an agent's)
fn users(i: &Item, user: &str) -> bool {
    i.assignee().is_some_and(|a| a == user)
}

fn kv(look: &Look, k: &str, v: String) -> Line<'static> {
    Line::from(vec![Span::styled(format!("  {:<26}", k), look.dim), Span::raw(v)])
}

fn head(text: &str) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), Style::default().add_modifier(ratatui::style::Modifier::BOLD)))
}

/// seconds per (weekday, hour) of `spans` (Mon = 0)
fn grid(spans: &[(i64, i64)], from: i64, to: i64) -> [[i64; 24]; 7] {
    let mut g = [[0i64; 24]; 7];
    for &(s, e) in spans {
        let (mut t, e) = (s.max(from), e.min(to));
        while t < e {
            let d = local(t);
            let next = (t - d.minute() as i64 * 60 - d.second() as i64 + 3600).min(e);
            g[d.weekday().num_days_from_monday() as usize][d.hour() as usize] += next - t;
            t = next;
        }
    }
    g
}

/// a weekday × hour heatmap, one char per hour
fn heat(g: &[[i64; 24]; 7], color: Color, look: &Look) -> Vec<Line<'static>> {
    let shades: Vec<char> = if crate::style::ascii() { " .:+#".chars().collect() } else { " ░▒▓█".chars().collect() };
    let max = g.iter().flatten().copied().max().unwrap_or(0).max(1);
    let names = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let mut out = vec![Line::from(Span::styled("     0     6     12    18   ", look.dim))];
    for (d, row) in g.iter().enumerate() {
        let cells: String = row.iter().map(|v| if *v == 0 { shades[0] } else { shades[1 + (*v * 3 / max) as usize] }).collect();
        out.push(Line::from(vec![Span::styled(format!("  {} ", names[d]), look.dim), Span::styled(cells, Style::default().fg(color))]));
    }
    out
}

/// work spans: (start, end, the user's); agents per session, so one
/// agent's parallel timers count once
fn spans(srcs: &[Src], user: &str) -> Vec<(i64, i64, bool)> {
    let mut v = vec![];
    for s in srcs {
        for i in s.store.get().values().filter(|i| users(i, user)) {
            v.extend(i.intervals().into_iter().map(|(a, b)| (a, b, true)));
        }
        v.extend(s.log.iter().filter(|l| l.who == user).filter_map(|l| l.span()).map(|(a, b)| (a, b, true)));
        let ids: Vec<&str> = s.store.get().keys().map(String::as_str).collect();
        v.extend(stats::agent_spans(s.store, &ids, s.log, user).into_iter().map(|(_, a, b)| (a, b, false)));
    }
    v
}

/// most at once of `spans`
fn most_at_once(spans: &[(i64, i64)]) -> usize {
    let mut edges: Vec<(i64, i32)> = spans.iter().flat_map(|(s, e)| [(*s, 1), (*e, -1)]).collect();
    edges.sort();
    let (mut cur, mut max) = (0i32, 0i32);
    for (_, d) in edges {
        cur += d;
        max = max.max(cur);
    }
    max as usize
}

pub fn rhythm(srcs: &[Src], user: &str, from: i64, to: i64, look: &Look) -> Vec<Line<'static>> {
    // the heatmap needs a few weeks to say something
    let wfrom = from.min(to - 28 * 86400);
    let all = spans(srcs, user);
    let you: Vec<(i64, i64)> = all.iter().filter(|s| s.2).map(|s| (s.0, s.1)).collect();
    let agents: Vec<(i64, i64)> = all.iter().filter(|s| !s.2).map(|s| (s.0, s.1)).collect();
    let mut out = vec![head("when you work (last 4 weeks)")];
    out.extend(heat(&grid(&you, wfrom, to), look.you, look));
    out.push(head("when agents work"));
    out.extend(heat(&grid(&agents, wfrom, to), look.agents, look));
    out.push(Line::default());
    out.push(head("records (all time)"));
    let mut per_day: BTreeMap<String, usize> = BTreeMap::new();
    for s in srcs {
        for i in s.store.get().values().filter(|i| i.status() == Some(Status::Done)) {
            if let Some(c) = i.closed() {
                *per_day.entry(local(c).format("%Y-%m-%d").to_string()).or_default() += 1;
            }
        }
    }
    if let Some((d, n)) = per_day.iter().max_by_key(|(d, n)| (**n, std::cmp::Reverse((*d).clone()))) {
        out.push(kv(look, "most done in a day", format!("{} on {}", n, d)));
    }
    let longest = merged(&you).iter().map(|(s, e)| e - s).max().unwrap_or(0);
    out.push(kv(look, "longest session", hm(longest)));
    out.push(kv(look, "most agents at once", most_at_once(&agents).to_string()));
    let mut active: Vec<String> = all.iter().map(|s| local(s.0).format("%Y-%m-%d").to_string()).chain(per_day.keys().cloned()).collect();
    active.sort();
    active.dedup();
    let mut best = 0;
    let mut run = 0;
    let mut prev: Option<chrono::NaiveDate> = None;
    for d in active.iter().filter_map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) {
        run = if prev.is_some_and(|p| d - p == chrono::Duration::days(1)) { run + 1 } else { 1 };
        best = best.max(run);
        prev = Some(d);
    }
    out.push(kv(look, "longest streak", format!("{} d", best)));
    out.push(Line::default());
    out.push(head("your time in this period"));
    let kind = |k: &str| -> i64 {
        let v: Vec<(i64, i64)> = srcs.iter().flat_map(|s| s.log.iter()).filter(|l| l.who == user && l.kind == k).filter_map(|l| l.span()).collect();
        merged(&v).iter().map(|(s, e)| (e.min(&to) - s.max(&from)).max(0)).sum()
    };
    out.push(kv(look, "chatting with agents", hm(kind("chat"))));
    out.push(kv(look, "in the form/dashboard", hm(kind("cake"))));
    out
}

fn merged(spans: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut v: Vec<(i64, i64)> = spans.iter().copied().filter(|(s, e)| e > s).collect();
    v.sort();
    let mut out: Vec<(i64, i64)> = vec![];
    for (s, e) in v {
        match out.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => out.push((s, e)),
        }
    }
    out
}

fn goal_name(store: &Store, id: &str) -> String {
    stats::goal_of(store, id).map_or("other".to_string(), |g| g.id().clone())
}

pub fn flow(srcs: &[Src], user: &str, from: i64, to: i64, look: &Look) -> Vec<Line<'static>> {
    let mut out = vec![head("created → done, median (closed in this period)")];
    let mut cycle: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut answers = vec![];
    let mut funnel: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    let mut inbox: Vec<(i64, String)> = vec![];
    for s in srcs {
        let items: Vec<&Item> = s.store.get().values().collect();
        for i in &items {
            let g = goal_name(s.store, i.id());
            let slot = match i.status() {
                Some(Status::Inbox) => 0,
                Some(Status::Task | Status::Doing | Status::Review) => 1,
                Some(Status::Done) => 2,
                Some(Status::Dropped) => 3,
                None => 4,
            };
            funnel.entry(g.clone()).or_default()[slot] += 1;
            if i.status() == Some(Status::Inbox) {
                inbox.push((to - i.timestamp(), i.title()));
            }
            let Some(c) = i.closed().filter(|c| *c >= from && *c <= to) else { continue };
            if i.status() == Some(Status::Done) {
                cycle.entry(g).or_default().push(c - i.timestamp());
            }
            if users(i, user) && items.iter().any(|o| o.blocked_by().contains(i.id())) {
                answers.push(c - i.timestamp());
            }
        }
    }
    if cycle.is_empty() {
        out.push(Line::from(Span::styled("  nothing closed yet", look.dim)));
    }
    for (g, v) in &cycle {
        out.push(kv(look, g, format!("{}  ({} done)", days(median(v.clone()).unwrap_or(0)), v.len())));
    }
    out.push(Line::default());
    out.push(head("inbox"));
    inbox.sort();
    out.push(kv(look, "waiting", format!("{} · median {}", inbox.len(), days(median(inbox.iter().map(|x| x.0).collect()).unwrap_or(0)))));
    if let Some((age, title)) = inbox.last() {
        out.push(kv(look, "oldest", format!("{} · {}", days(*age), title)));
    }
    out.push(Line::default());
    out.push(head("your answers to agents (this period)"));
    if answers.is_empty() {
        out.push(Line::from(Span::styled("  none", look.dim)));
    } else {
        out.push(kv(look, "median wait", days(median(answers.clone()).unwrap_or(0))));
        out.push(kv(look, "longest wait", days(answers.iter().copied().max().unwrap_or(0))));
    }
    out.push(Line::default());
    out.push(head("per goal now:   inbox  open  done  dropped  notes"));
    for (g, f) in &funnel {
        out.push(Line::from(vec![Span::styled(format!("  {:<14}", g), look.dim), Span::raw(format!("{:>6} {:>5} {:>5} {:>8} {:>6}", f[0], f[1], f[2], f[3], f[4]))]));
    }
    out
}

pub fn agents(srcs: &[Src], user: &str, from: i64, to: i64, look: &Look) -> Vec<Line<'static>> {
    let mut out = vec![head("estimates vs real time (done tasks)")];
    let mut ratios = vec![];
    let mut n = stats::Numbers::default();
    for s in srcs {
        n.add(&stats::window(s.store, s.log, user, from, to));
        for i in s.store.get().values() {
            if let (Some(e), true) = (i.estimate(), i.status() == Some(Status::Done)) {
                if i.tracked_seconds() >= 600 {
                    ratios.push(i.tracked_seconds() * 100 / e.max(1));
                }
            }
        }
    }
    match median(ratios.clone()) {
        Some(r) => out.push(kv(look, "real / estimate", format!("{}%  ({} tasks)", r, ratios.len()))),
        None => out.push(Line::from(Span::styled("  no done task with an estimate yet", look.dim))),
    }
    out.push(Line::default());
    out.push(kv(look, "agent time per done item", if n.done > 0 { hm(n.agents / n.done as i64) } else { "-".into() }));
    out.push(Line::default());
    out.push(head("parallel"));
    let agent_spans: Vec<(i64, i64)> = spans(srcs, user).into_iter().filter(|s| !s.2).map(|s| (s.0.max(from), s.1.min(to))).filter(|(a, b)| b > a).collect();
    let busy: i64 = merged(&agent_spans).iter().map(|(a, b)| b - a).sum();
    let sum: i64 = agent_spans.iter().map(|(a, b)| b - a).sum();
    out.push(kv(look, "agents at once, avg", if busy > 0 { format!("{:.1}", sum as f64 / busy as f64) } else { "-".into() }));
    out.extend(configured(srcs, user, from, to, look));
    out
}

/// the sections of config stats.sections
pub fn configured(srcs: &[Src], user: &str, from: i64, to: i64, look: &Look) -> Vec<Line<'static>> {
    let sections = &stats::conf().sections;
    if sections.is_empty() {
        return vec![];
    }
    let mut n = stats::Numbers::default();
    for s in srcs {
        n.add(&stats::window(s.store, s.log, user, from, to));
    }
    let mut out = vec![];
    for sec in sections {
        out.push(Line::default());
        out.push(head(&sec.title));
        for row in &sec.rows {
            if let (Some(top), Some(by)) = (row.top, &row.by) {
                let idx = stats::conf().sums.iter().position(|s| &s.name == by);
                let mut best: Vec<(f64, String)> = vec![];
                for s in srcs {
                    for i in s.store.get().values() {
                        let v = idx.and_then(|k| stats::item_sums(s.store, i).get(k).copied()).unwrap_or(0.0) * stats::share(i, from, to);
                        if v > 0.0 {
                            best.push((v, i.title()));
                        }
                    }
                }
                best.sort_by(|a, b| b.0.total_cmp(&a.0));
                for (v, title) in best.into_iter().take(top) {
                    out.push(Line::from(vec![Span::styled(format!("  {:>7}  ", stats::human(v)), look.dim), Span::raw(title)]));
                }
            } else if let Some(f) = &row.value {
                let v = n.eval(f);
                out.push(kv(look, &row.label, if v.is_finite() { stats::human(v) } else { "-".into() }));
            }
        }
    }
    out
}

pub fn fun(srcs: &[Src], user: &str, from: i64, to: i64, look: &Look) -> Vec<Line<'static>> {
    let mut n = stats::Numbers::default();
    for s in srcs {
        n.add(&stats::window(s.store, s.log, user, from, to));
    }
    let mut out = vec![head("words in items: you vs agents")];
    let (mut you, mut them) = (0usize, 0usize);
    for s in srcs {
        for i in s.store.get().values() {
            let w = i.content().split_whitespace().count();
            if i.by().is_some_and(|b| b == user) {
                you += w;
            } else {
                them += w;
            }
        }
    }
    let total = (you + them).max(1);
    let bar = 30;
    let mine = you * bar / total;
    out.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("█".repeat(mine), Style::default().fg(look.you)),
        Span::styled("█".repeat(bar - mine), Style::default().fg(look.agents)),
        Span::styled(format!("  you {} · agents {}", you, them), look.dim),
    ]));
    out.push(Line::default());
    out.push(head("the cake"));
    let total_time = n.you + n.agents;
    out.push(kv(look, "baking time", hm(total_time)));
    out.push(kv(look, "slices eaten", format!("{} done", n.done)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_land_in_their_weekday_and_hour() {
        let mon9 = Local.with_ymd_and_hms(2026, 11, 16, 9, 30, 0).unwrap().timestamp();
        let g = grid(&[(mon9, mon9 + 3600)], mon9 - 86400, mon9 + 86400);
        assert_eq!((g[0][9], g[0][10]), (1800, 1800), "9:30-10:30 splits over two hours");
    }

    #[test]
    fn overlaps_merge_and_count() {
        assert_eq!(merged(&[(0, 10), (5, 20), (30, 40)]), vec![(0, 20), (30, 40)]);
        assert_eq!(most_at_once(&[(0, 10), (5, 20), (6, 7), (30, 40)]), 3);
        assert_eq!(median(vec![5, 1, 3]), Some(3));
        assert_eq!(median(vec![]), None);
    }
}
