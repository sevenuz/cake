//! who worked when: lanes of time bars for the dashboard's `g` overlay.
//! G1 lanes per session (you, each agent per store, packed so parallel
//! work gets its own lane, and what waited on you), G2 lanes per store
//! shaded by how many worked at once. Pure: stores and log lines come in,
//! styled lines go out.

use crate::item::Item;
use crate::log::Line as LogLine;
use crate::store::Store;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fill {
    /// working: chat, an agent's timer
    Solid,
    /// the user in the cake form or dashboard
    Light,
    /// an agent waits on the user (an open question)
    Wait,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub start: i64,
    pub end: i64,
    /// the store, for its color
    pub store: usize,
    pub fill: Fill,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub name: String,
    pub bars: Vec<Bar>,
    /// shaded by how many bars overlap (░▒▓█) instead of one block
    pub stacked: bool,
}

/// one store's input
pub struct Src<'a> {
    pub name: &'a str,
    pub store: &'a Store,
    pub log: &'a [LogLine],
}

fn overlaps(s: i64, e: i64, from: i64, to: i64) -> bool {
    s < to && e > from
}

/// spans of questions to the user that blocked other work: created until
/// answered (or now)
fn waits(store: &Store, user: &str, now: i64) -> Vec<(i64, i64)> {
    let items: Vec<&Item> = store.get().values().collect();
    items
        .iter()
        .filter(|q| q.assignee().is_some_and(|a| a == user))
        .filter(|q| items.iter().any(|o| o.blocked_by().contains(q.id())))
        .map(|q| (q.timestamp(), q.closed().unwrap_or(now)))
        .collect()
}

/// G1: you, the agents of each store (one lane, shaded by how many
/// sessions worked at once), and what waited on you
pub fn sessions(srcs: &[Src], user: &str, from: i64, to: i64) -> Vec<Lane> {
    let mut you = vec![];
    let mut out = vec![];
    let mut waited = vec![];
    for (n, s) in srcs.iter().enumerate() {
        for l in s.log.iter().filter(|l| l.who == user) {
            if let Some((a, b)) = l.span().filter(|(a, b)| overlaps(*a, *b, from, to)) {
                you.push(Bar { start: a, end: b, store: n, fill: if l.kind == "cake" { Fill::Light } else { Fill::Solid } });
            }
        }
        for i in s.store.get().values().filter(|i| i.assignee().is_some_and(|a| a == user)) {
            you.extend(i.intervals().into_iter().filter(|(a, b)| overlaps(*a, *b, from, to)).map(|(a, b)| Bar { start: a, end: b, store: n, fill: Fill::Solid }));
        }
        // per session: several timers of one agent count once
        let ids: Vec<&str> = s.store.get().keys().map(String::as_str).collect();
        let agents: Vec<Bar> = crate::stats::agent_spans(s.store, &ids, s.log, user)
            .into_iter()
            .filter(|(_, a, b)| overlaps(*a, *b, from, to))
            .map(|(_, a, b)| Bar { start: a, end: b, store: n, fill: Fill::Solid })
            .collect();
        if !agents.is_empty() {
            out.push(Lane { name: format!("agents · {}", s.name), bars: agents, stacked: true });
        }
        waited.extend(waits(s.store, user, to).into_iter().filter(|(a, b)| overlaps(*a, *b, from, to)).map(|(a, b)| Bar { start: a, end: b, store: n, fill: Fill::Wait }));
    }
    let mut lanes = vec![Lane { name: "you".into(), bars: you, stacked: false }];
    lanes.extend(out);
    if !waited.is_empty() {
        lanes.push(Lane { name: "waits on you".into(), bars: waited, stacked: false });
    }
    lanes
}

/// G3: one lane per item worked in [from, to]: its timers, and the
/// user's chat time on it (light), first started first
pub fn items(srcs: &[Src], user: &str, from: i64, to: i64) -> Vec<Lane> {
    let mut out: Vec<Lane> = vec![];
    for (n, s) in srcs.iter().enumerate() {
        let mut by_item: std::collections::BTreeMap<String, Vec<Bar>> = Default::default();
        for i in s.store.get().values() {
            for (a, b) in i.intervals().into_iter().filter(|(a, b)| overlaps(*a, *b, from, to)) {
                by_item.entry(i.id().clone()).or_default().push(Bar { start: a, end: b, store: n, fill: Fill::Solid });
            }
        }
        for l in s.log.iter().filter(|l| l.who == user) {
            if let (Some(id), Some((a, b))) = (&l.item, l.span().filter(|(a, b)| overlaps(*a, *b, from, to))) {
                by_item.entry(id.clone()).or_default().push(Bar { start: a, end: b, store: n, fill: Fill::Light });
            }
        }
        out.extend(by_item.into_iter().map(|(name, bars)| Lane { name, bars, stacked: false }));
    }
    out.sort_by_key(|l| (l.bars.iter().map(|b| b.start).min().unwrap_or(0), l.name.clone()));
    out
}

/// G2: one lane per store with everything that ran there; render shades
/// it by how many bars overlap
pub fn stores(srcs: &[Src], user: &str, from: i64, to: i64) -> Vec<Lane> {
    let g1 = sessions(srcs, user, from, to);
    (0..srcs.len())
        .map(|n| Lane { name: srcs[n].name.to_string(), bars: g1.iter().flat_map(|l| l.bars.iter().filter(|b| b.store == n).cloned()).collect(), stacked: true })
        .collect()
}

/// (most at once, average at once while anyone worked, seconds agents
/// waited on the user)
pub fn summary(lanes: &[Lane], from: i64, to: i64) -> (usize, f64, i64) {
    let work: Vec<&Bar> = lanes.iter().flat_map(|l| &l.bars).filter(|b| b.fill != Fill::Wait).collect();
    let mut edges: Vec<(i64, i32)> = work.iter().flat_map(|b| [(b.start.max(from), 1), (b.end.min(to), -1)]).collect();
    edges.sort();
    let (mut cur, mut max, mut busy, mut weighted, mut last) = (0i32, 0i32, 0i64, 0i64, from);
    for (t, d) in edges {
        if cur > 0 {
            busy += t - last;
            weighted += (t - last) * cur as i64;
        }
        cur += d;
        max = max.max(cur);
        last = t;
    }
    let waited = lanes.iter().flat_map(|l| &l.bars).filter(|b| b.fill == Fill::Wait).map(|b| b.end.min(to) - b.start.max(from)).sum();
    (max as usize, if busy > 0 { weighted as f64 / busy as f64 } else { 0.0 }, waited)
}

/// the bars touching each of `width` columns of [from, to] (a short bar
/// still shows)
fn columns(bars: &[&Bar], from: i64, to: i64, width: usize) -> Vec<Vec<usize>> {
    let span = (to - from).max(1) as f64;
    let at = |c: usize| from + (c as f64 / width as f64 * span) as i64;
    (0..width)
        .map(|c| {
            let (s, e) = (at(c), at(c + 1));
            bars.iter().enumerate().filter(|(_, b)| b.start < e && b.end > s).map(|(n, _)| n).collect()
        })
        .collect()
}

/// the time axis: labels at round hours (or days) above the columns
pub fn axis(from: i64, to: i64, width: usize) -> String {
    let span = (to - from).max(1);
    let hour = 3600;
    let step = [hour, 2 * hour, 3 * hour, 6 * hour, 12 * hour, 24 * hour, 48 * hour]
        .into_iter()
        .find(|s| (span / s) as usize * 6 <= width)
        .unwrap_or(7 * 24 * hour);
    let mut out = vec![' '; width];
    let local = |t: i64| chrono::TimeZone::timestamp_opt(&chrono::Local, t, 0).single();
    let offset = local(from).map_or(0, |t| chrono::Offset::fix(t.offset()).local_minus_utc() as i64);
    let mut t = ((from + offset) / step + 1) * step - offset;
    while t < to {
        let c = ((t - from) as f64 / span as f64 * width as f64) as usize;
        // midnight gets the day when the axis spans days
        let label = local(t).map_or(String::new(), |d| {
            if step >= 24 * hour || (span > 24 * hour && chrono::Timelike::hour(&d) == 0) { d.format("%a %d").to_string() } else { d.format("%H").to_string() }
        });
        if c + label.len() <= width && out[c.saturating_sub(1)..c + label.len()].iter().all(|x| *x == ' ') {
            for (k, ch) in label.chars().enumerate() {
                out[c + k] = ch;
            }
        }
        t += step;
    }
    out.into_iter().collect()
}

/// a lane as `width` cells: G1 bars in their store's color, G2 (`shade`)
/// by how many at once
pub fn render(lane: &Lane, from: i64, to: i64, width: usize, colors: &[Color], shade: bool, ascii: bool) -> Line<'static> {
    let shade = shade || lane.stacked;
    let bars: Vec<&Bar> = lane.bars.iter().collect();
    let cols = columns(&bars, from, to, width);
    let (solid, light, wait) = if ascii { ('#', '+', '.') } else { ('█', '▒', '·') };
    let shades: Vec<char> = if ascii { vec!['.', ':', '+', '#'] } else { vec!['░', '▒', '▓', '█'] };
    let mut spans: Vec<Span<'static>> = vec![];
    for c in cols {
        let work: Vec<&&Bar> = c.iter().map(|n| &bars[*n]).filter(|b| b.fill != Fill::Wait).collect();
        let waits = c.iter().any(|n| bars[*n].fill == Fill::Wait);
        let (ch, color) = match work.first() {
            None if waits => (if shade { '⁚' } else { wait }, None),
            None => (' ', None),
            Some(b) if shade => (shades[(work.len() - 1).min(3)], Some(colors[b.store % colors.len()])),
            Some(b) => (if work.iter().all(|b| b.fill == Fill::Light) { light } else { solid }, Some(colors[b.store % colors.len()])),
        };
        let ch = if ascii && ch == '⁚' { ':' } else { ch };
        spans.push(match color {
            Some(c) => Span::styled(ch.to_string(), Style::default().fg(c)),
            None => Span::raw(ch.to_string()),
        });
    }
    Line::from(spans)
}

/// an open task of a goal, scheduled from now: after its blockers
#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    pub id: String,
    pub start: i64,
    pub end: i64,
    /// a question to the user (no duration of its own, shown as ◆)
    pub question: bool,
    pub on_path: bool,
    /// the duration is a guess from history (no estimate)
    pub guess: bool,
}

/// open tasks below `goal`, each after its blockers (earliest start from
/// `now`, its estimate long, else the history median, else 1 h; questions
/// take no time); the critical path is the chain that ends last
pub fn plan(store: &Store, goal: &str, user: &str, now: i64) -> Vec<Planned> {
    use std::collections::{HashMap, HashSet};
    let mut ids: Vec<String> = vec![];
    let mut todo = vec![goal.to_string()];
    let mut seen = HashSet::new();
    while let Some(id) = todo.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(i) = store.get_item(&id) {
            if id != goal && i.is_open() && i.status().is_some() {
                ids.push(id.clone());
            }
            todo.extend(i.children().iter().cloned());
        }
    }
    ids.sort();
    let set: HashSet<&String> = ids.iter().collect();
    let question = |id: &str| store.get_item(id).is_some_and(|i| i.assignee().is_some_and(|a| a == user) && store.get().values().any(|o| o.blocked_by().iter().any(|b| b == id)));
    let length = |id: &str| -> (i64, bool) {
        if question(id) {
            return (0, false);
        }
        store.expected(id).unwrap_or((3600, true))
    };
    // earliest end of each, through its blockers inside the goal
    fn end(id: &str, store: &Store, set: &HashSet<&String>, length: &dyn Fn(&str) -> (i64, bool), now: i64, memo: &mut HashMap<String, (i64, Option<String>)>, depth: usize) -> i64 {
        if let Some((e, _)) = memo.get(id) {
            return *e;
        }
        let mut start = now;
        let mut before = None;
        if depth < 64 {
            for b in store.get_item(id).map(|i| i.blocked_by().clone()).unwrap_or_default() {
                if set.contains(&b) {
                    let e = end(&b, store, set, length, now, memo, depth + 1);
                    if e > start {
                        start = e;
                        before = Some(b.clone());
                    }
                }
            }
        }
        let e = start + length(id).0;
        memo.insert(id.to_string(), (e, before));
        e
    }
    let mut memo = HashMap::new();
    for id in &ids {
        end(id, store, &set, &length, now, &mut memo, 0);
    }
    // the path: back from the latest end
    let mut path = HashSet::new();
    let mut cur = ids.iter().max_by_key(|id| (memo[*id].0, std::cmp::Reverse((*id).clone()))).cloned();
    while let Some(c) = cur {
        path.insert(c.clone());
        cur = memo[&c].1.clone();
    }
    let mut out: Vec<Planned> = ids
        .iter()
        .map(|id| {
            let (len, guess) = length(id);
            let e = memo[id].0;
            Planned { id: id.clone(), start: e - len, end: e, question: question(id), on_path: path.contains(id), guess }
        })
        .collect();
    out.sort_by_key(|p| (p.start, p.end, p.id.clone()));
    out
}

/// goals (notes tagged goal) with a deadline: (id, deadline end of day)
pub fn goals_with_deadline(store: &Store) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> = store
        .get()
        .values()
        .filter(|i| i.status().is_none() && i.tags().iter().any(|t| t == "goal"))
        .filter_map(|i| i.deadline().and_then(|d| crate::util::date_timestamp(d)).map(|t| (i.id().clone(), t)))
        .collect();
    v.sort_by_key(|(id, t)| (*t, id.clone()));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Status;

    fn store() -> Store {
        Store::new(std::path::Path::new("/definitely/does/not/exist/cake-gantt-test")).unwrap()
    }

    fn task(id: &str, who: &str, time: Vec<i64>) -> Item {
        let mut i = Item::new(id.into(), vec![], None, vec![]);
        i.set_content(format!("# {}\n", id));
        i.set_status(Status::Task).unwrap();
        i.set_assignee(Some(who.into()));
        i.set_timetrack(time);
        i
    }

    #[test]
    fn agents_share_one_lane_shaded_by_how_many_ran() {
        let h = 3600;
        let mut s = store();
        s.add(task("a", "claude-code", vec![0, 2 * h])).unwrap();
        s.add(task("b", "claude-code", vec![h, 3 * h])).unwrap();
        s.add(task("c", "claude-code", vec![3 * h, 4 * h])).unwrap();
        let mut blocked = task("blocked", "claude-code", vec![]);
        blocked.set_blocked_by(vec!["q".into()]);
        s.add(blocked).unwrap();
        let mut q = task("q", "jul", vec![]);
        q.set_status(Status::Done).unwrap();
        s.add(q).unwrap();
        let log = vec![LogLine::new(0, h, "jul", "chat"), LogLine::new(h, h + 600, "jul", "cake")];
        let src = Src { name: "cake", store: &s, log: &log };
        let lanes = sessions(&[src], "jul", 0, 4 * h);
        let names: Vec<&str> = lanes.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["you", "agents · cake"], "the question was asked outside the window");
        assert_eq!(render(&lanes[1], 0, 4 * h, 4, &[Color::Reset], false, false).to_string(), "░▒░░", "a, a+b, b, c");
        let (max, avg, _) = summary(&lanes, 0, 4 * h);
        assert_eq!(max, 3, "you, a and b at 1h");
        assert!(avg > 1.0);
        let line = render(&lanes[0], 0, 4 * h, 8, &[Color::Reset], false, false).to_string();
        assert_eq!(line, "██▒     ", "chat 0-1h, the form 1h-1h10");
    }

    #[test]
    fn items_get_a_lane_each_with_the_users_chat_time() {
        let h = 3600;
        let mut s = store();
        s.add(task("a", "cc", vec![h, 2 * h])).unwrap();
        s.add(task("b", "cc", vec![0, h])).unwrap();
        s.add(task("old", "cc", vec![-9 * h, -8 * h])).unwrap();
        let mut l = LogLine::new(h, 3 * h, "jul", "chat");
        l.item = Some("a".into());
        let log = vec![l];
        let lanes = items(&[Src { name: "cake", store: &s, log: &log }], "jul", 0, 4 * h);
        let names: Vec<&str> = lanes.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["b", "a"], "first started first, nothing outside the window");
        assert!(!lanes[0].stacked);
        assert_eq!(render(&lanes[1], 0, 4 * h, 4, &[Color::Reset], false, false).to_string(), " █▒ ", "timer, then chat only");
    }

    #[test]
    fn stores_shade_by_how_many_at_once() {
        let h = 3600;
        let mut s = store();
        s.add(task("a", "x", vec![0, 2 * h])).unwrap();
        s.add(task("b", "y", vec![h, 2 * h])).unwrap();
        let src = Src { name: "cake", store: &s, log: &[] };
        let lanes = stores(&[src], "jul", 0, 2 * h);
        assert_eq!(render(&lanes[0], 0, 2 * h, 4, &[Color::Reset], true, false).to_string(), "░░▒▒");
    }

    #[test]
    fn the_critical_path_is_the_chain_that_ends_last() {
        let h = 3600;
        let mut s = store();
        let mut goal = Item::new("g".into(), vec![], None, vec!["goal".into()]);
        goal.set_content("# g\n".into());
        s.add(goal).unwrap();
        let mut add = |id: &str, est: &str, blocked: Vec<&str>, who: &str| {
            let mut i = task(id, who, vec![]);
            i.set_parent(Some("g".into()));
            if !est.is_empty() {
                i.set_estimate(Some(est.into())).unwrap();
            }
            i.set_blocked_by(blocked.into_iter().map(String::from).collect());
            s.add(i).unwrap();
        };
        add("a", "2h", vec![], "cc");
        add("b", "3h", vec!["a", "q"], "cc");
        add("side", "1h", vec![], "cc");
        add("q", "", vec![], "jul");
        let p = plan(&s, "g", "jul", 0);
        let get = |id: &str| p.iter().find(|x| x.id == id).unwrap().clone();
        assert_eq!((get("b").start, get("b").end), (2 * h, 5 * h));
        assert!(get("a").on_path && get("b").on_path && !get("side").on_path);
        assert!(get("q").question && get("q").end == 0);
    }

    #[test]
    fn axis_labels_round_hours() {
        let from = chrono::TimeZone::with_ymd_and_hms(&chrono::Local, 2026, 11, 15, 8, 30, 0).unwrap().timestamp();
        let a = axis(from, from + 8 * 3600, 48);
        assert!(a.contains("09") && a.contains("12"), "{:?}", a);
    }
}
