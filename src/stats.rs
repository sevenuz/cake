//! numbers for a period: the user's time (session log and their own
//! timers), agents' time (timers of delegated items), tokens and their
//! footprint, done items; per store and per goal, with buckets for a
//! sparkline. Pure: everything comes in as arguments, nothing is read here
//! except through `of_store`.

use crate::item::{Item, Status};
use crate::log::Line;
use crate::store::Store;
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Period {
    Day,
    /// today and the 6 days before
    Days7,
    Week,
    Month,
    Year,
}

impl Period {
    pub fn parse(s: &str) -> Option<Period> {
        match s {
            "day" | "d" | "today" => Some(Period::Day),
            "7d" | "7days" | "7 days" => Some(Period::Days7),
            "week" | "w" => Some(Period::Week),
            "month" | "m" => Some(Period::Month),
            "year" | "y" => Some(Period::Year),
            _ => None,
        }
    }

    /// day → week → month → year → day
    pub fn next(self) -> Period {
        match self {
            Period::Day => Period::Days7,
            Period::Days7 => Period::Week,
            Period::Week => Period::Month,
            Period::Month => Period::Year,
            Period::Year => Period::Day,
        }
    }

    /// what one sparkline mark stands for
    pub fn bucket(self) -> &'static str {
        match self {
            Period::Day => "per hour",
            Period::Year => "per month",
            _ => "per day",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Period::Day => "day",
            Period::Days7 => "7 days",
            Period::Week => "week",
            Period::Month => "month",
            Period::Year => "year",
        }
    }

    /// (start, bucket starts) of the period that contains `now`, local time:
    /// today in hours, this week (from Monday) and month in days, this year
    /// in months
    pub fn range(self, now: i64) -> (i64, Vec<i64>) {
        let today = Local.timestamp_opt(now, 0).single().map_or(NaiveDate::default(), |t| t.date_naive());
        let at = |d: NaiveDate| d.and_hms_opt(0, 0, 0).and_then(|t| Local.from_local_datetime(&t).earliest()).map_or(0, |t| t.timestamp());
        let days = |from: NaiveDate| -> Vec<i64> { from.iter_days().take_while(|d| *d <= today).map(at).collect() };
        match self {
            Period::Day => {
                let start = at(today);
                (start, (0..24).map(|h| start + h * 3600).filter(|t| *t < now).collect())
            }
            Period::Days7 => {
                let from = today - Duration::days(6);
                (at(from), days(from))
            }
            Period::Week => {
                let from = today - Duration::days(today.weekday().num_days_from_monday() as i64);
                (at(from), days(from))
            }
            Period::Month => {
                let from = today.with_day(1).unwrap();
                (at(from), days(from))
            }
            Period::Year => {
                let from = today.with_ordinal(1).unwrap();
                let months = (1..=today.month()).filter_map(|m| NaiveDate::from_ymd_opt(today.year(), m, 1)).map(at).collect();
                (at(from), months)
            }
        }
    }
}

/// seconds of [s, e] inside [from, to]
fn clip(s: i64, e: i64, from: i64, to: i64) -> i64 {
    (e.min(to) - s.max(from)).max(0)
}

/// the spans clipped to [from, to], overlaps counted once
fn union_in(spans: &[(i64, i64)], from: i64, to: i64) -> i64 {
    let mut v: Vec<(i64, i64)> = spans.iter().map(|(s, e)| ((*s).max(from), (*e).min(to))).filter(|(s, e)| e > s).collect();
    v.sort();
    let mut total = 0;
    let mut cur: Option<(i64, i64)> = None;
    for (s, e) in v {
        cur = match cur {
            Some((cs, ce)) if s <= ce => Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                total += ce - cs;
                Some((s, e))
            }
            None => Some((s, e)),
        };
    }
    total + cur.map_or(0, |(s, e)| e - s)
}

#[derive(Serialize, Default, Debug, Clone, PartialEq)]
pub struct Numbers {
    /// the user's time, overlaps once
    pub you: i64,
    /// agents' time, parallel agents add up
    pub agents: i64,
    pub tokens: u64,
    pub wh: f64,
    pub co2_g: f64,
    pub water_l: f64,
    /// closed as done in the period
    pub done: usize,
}

impl Numbers {
    pub fn add(&mut self, o: &Numbers) {
        self.you += o.you;
        self.agents += o.agents;
        self.tokens += o.tokens;
        self.wh += o.wh;
        self.co2_g += o.co2_g;
        self.water_l += o.water_l;
        self.done += o.done;
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Goal {
    /// "" for items under no goal
    pub id: String,
    pub title: String,
    #[serde(flatten)]
    pub numbers: Numbers,
}

#[derive(Serialize, Debug, Clone)]
pub struct StoreStats {
    pub period: &'static str,
    pub from: i64,
    pub to: i64,
    #[serde(flatten)]
    pub numbers: Numbers,
    pub goals: Vec<Goal>,
    /// (bucket start, you + agents seconds) for a sparkline
    pub buckets: Vec<(i64, i64)>,
    /// open items past their deadline or due date
    pub overdue: usize,
    pub review: usize,
    /// open tasks of the user that block other items
    pub questions: usize,
    /// days in a row up to today with time or a done item
    pub streak: usize,
}

/// the nearest note tagged goal at or above `id`
fn goal_of<'a>(store: &'a Store, id: &str) -> Option<&'a Item> {
    let mut seen = std::collections::HashSet::new();
    let mut cur = store.get_item(id);
    while let Some(i) = cur {
        if !seen.insert(i.id().clone()) {
            break;
        }
        if i.status().is_none() && i.tags().iter().any(|t| t == "goal") {
            return Some(i);
        }
        cur = i.parents().first().and_then(|p| store.get_item(p));
    }
    None
}

/// the user's own work: assigned to them (else an agent's)
fn users(i: &Item, user: &str) -> bool {
    i.assignee().is_some_and(|a| a == user)
}

/// numbers of one item set in [from, to]: `log` the user's lines
fn numbers(store: &Store, ids: &[&str], log: &[(i64, i64)], user: &str, from: i64, to: i64) -> Numbers {
    let mut n = Numbers::default();
    let mut you: Vec<(i64, i64)> = log.to_vec();
    for id in ids {
        let Some(i) = store.get_item(id) else { continue };
        let spans = i.intervals();
        if users(i, user) {
            you.extend(spans.iter().copied());
        } else {
            n.agents += spans.iter().map(|(s, e)| clip(*s, *e, from, to)).sum::<i64>();
        }
        // tokens by the share of the item's time in the period; without
        // time, by when it was closed (or created)
        let total: i64 = spans.iter().map(|(s, e)| e - s).sum();
        let share = if total > 0 {
            spans.iter().map(|(s, e)| clip(*s, *e, from, to)).sum::<i64>() as f64 / total as f64
        } else {
            let at = i.closed().unwrap_or(i.timestamp());
            if at >= from && at <= to { 1.0 } else { 0.0 }
        };
        if share > 0.0 && !i.usage().is_empty() {
            let (wh, co2, water) = crate::footprint::amounts(i.usage());
            n.tokens += (i.usage().values().map(|u| u.total()).sum::<u64>() as f64 * share).round() as u64;
            n.wh += wh * share;
            n.co2_g += co2 * share;
            n.water_l += water * share;
        }
        if i.status() == Some(Status::Done) && i.closed().is_some_and(|c| c >= from && c <= to) {
            n.done += 1;
        }
    }
    n.you = union_in(&you, from, to);
    n
}

/// the numbers of `store` for `period` up to `now`; `log` its session log
/// lines, `user` whose time is "you"
pub fn of_store(store: &Store, log: &[Line], user: &str, period: Period, now: i64) -> StoreStats {
    let (from, starts) = period.range(now);
    let all: Vec<&str> = store.get().keys().map(String::as_str).collect();
    let mine: Vec<(i64, i64, Option<String>)> =
        log.iter().filter(|l| l.who == user).filter_map(|l| l.span().map(|(s, e)| (s, e, l.item.clone()))).collect();
    let spans = |pick: &dyn Fn(&Option<String>) -> bool| -> Vec<(i64, i64)> { mine.iter().filter(|(_, _, i)| pick(i)).map(|(s, e, _)| (*s, *e)).collect() };
    let total = numbers(store, &all, &spans(&|_| true), user, from, now);

    // per goal: items under it, log lines of its items; the rest is ""
    let goal_id = |id: &str| goal_of(store, id).map_or(String::new(), |g| g.id().clone());
    let mut by_goal: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for id in &all {
        by_goal.entry(goal_id(id)).or_default().push(id);
    }
    let mut goals: Vec<Goal> = by_goal
        .iter()
        .map(|(g, ids)| {
            let lines = spans(&|i| i.as_deref().map_or(g.is_empty(), |i| goal_id(i) == *g));
            let title = store.get_item(g).map_or("other".to_string(), |i| i.title());
            Goal { id: g.clone(), title, numbers: numbers(store, ids, &lines, user, from, now) }
        })
        .filter(|g| g.numbers != Numbers::default())
        .collect();
    goals.sort_by_key(|g| -(g.numbers.you + g.numbers.agents));

    let mut ends: Vec<i64> = starts.iter().skip(1).copied().collect();
    ends.push(now);
    let buckets = starts
        .iter()
        .zip(&ends)
        .map(|(s, e)| {
            let n = numbers(store, &all, &spans(&|_| true), user, *s, *e);
            (*s, n.you + n.agents)
        })
        .collect();

    let today = crate::util::format_timestamp(now)[..10].to_string();
    let items: Vec<&Item> = store.get().values().collect();
    let overdue = items
        .iter()
        .filter(|i| i.is_open() && [i.deadline(), i.due()].into_iter().flatten().any(|d| d[..10.min(d.len())] < *today))
        .count();
    let review = items.iter().filter(|i| i.status() == Some(Status::Review)).count();
    let questions = items
        .iter()
        .filter(|i| i.is_open() && users(i, user) && items.iter().any(|o| o.is_open() && o.blocked_by().contains(i.id())))
        .count();

    // streak: whole days back from today with time or a done item
    let mut streak = 0;
    let (mut day_start, _) = Period::Day.range(now);
    let mut day_end = now;
    loop {
        let n = numbers(store, &all, &spans(&|_| true), user, day_start, day_end);
        if n.you + n.agents == 0 && n.done == 0 {
            break;
        }
        streak += 1;
        day_end = day_start;
        day_start = Period::Day.range(day_start - 3600).0;
        if streak > 366 {
            break;
        }
    }

    StoreStats { period: period.name(), from, to: now, numbers: total, goals, buckets, overdue, review, questions, streak }
}

/// the numbers of all of `store` in [from, to]
pub fn window(store: &Store, log: &[Line], user: &str, from: i64, to: i64) -> Numbers {
    let all: Vec<&str> = store.get().keys().map(String::as_str).collect();
    let mine: Vec<(i64, i64)> = log.iter().filter(|l| l.who == user).filter_map(|l| l.span()).collect();
    numbers(store, &all, &mine, user, from, to)
}

/// you + agents seconds for each of the last `n` days (today last)
pub fn days(store: &Store, log: &[Line], user: &str, n: usize, now: i64) -> Vec<i64> {
    let mut out = vec![];
    let (mut start, _) = Period::Day.range(now);
    let mut end = now;
    for _ in 0..n {
        let x = window(store, log, user, start, end);
        out.push(x.you + x.agents);
        end = start;
        start = Period::Day.range(start - 3600).0;
    }
    out.reverse();
    out
}

/// sum of several stores' numbers
pub fn sum<'a>(stats: impl IntoIterator<Item = &'a StoreStats>) -> Numbers {
    let mut n = Numbers::default();
    for s in stats {
        n.add(&s.numbers);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Usage;

    fn store() -> Store {
        Store::new(std::path::Path::new("/definitely/does/not/exist/cake-stats-test")).unwrap()
    }

    fn item(id: &str, parent: Option<&str>) -> Item {
        let mut i = Item::new(id.into(), vec![], parent.map(String::from), vec![]);
        i.set_content(format!("# {}\n", id));
        i
    }

    #[test]
    fn clipping_and_union() {
        assert_eq!(clip(0, 100, 50, 200), 50);
        assert_eq!(clip(0, 10, 50, 200), 0);
        assert_eq!(union_in(&[(0, 100), (50, 150), (300, 400)], 0, 350), 200);
    }

    #[test]
    fn time_tokens_and_done_per_goal_in_the_period() {
        let now = Local.with_ymd_and_hms(2026, 11, 15, 18, 0, 0).unwrap().timestamp();
        let h = 3600;
        let mut s = store();
        let mut g = item("stats", None);
        g.append_tags(vec!["goal".into()]);
        s.add(g).unwrap();
        // an agent's task: 2 h yesterday, 1 h today; 900 tokens
        let mut a = item("agent-task", Some("stats"));
        a.set_status(Status::Done).unwrap();
        a.set_assignee(Some("claude-code".into()));
        a.set_timetrack(vec![now - 26 * h, now - 24 * h, now - 3 * h, now - 2 * h]);
        a.add_usage("m", Usage { input: 0, output: 900, cache_read: 0, cache_write: 0 });
        s.add(a).unwrap();
        // the user's own task without a goal: a running timer for 30 min
        let mut u = item("mine", None);
        u.set_status(Status::Doing).unwrap();
        u.set_assignee(Some("jul".into()));
        u.set_timetrack(vec![now - h / 2]);
        s.add(u).unwrap();
        // the user chatted 1 h about agent-task, overlapping its timer 30 min
        let mut l = Line::new(now - 3 * h - h / 2, now - 2 * h - h / 2, "jul", "chat");
        l.item = Some("agent-task".into());
        let st = of_store(&s, &[l], "jul", Period::Day, now);
        assert_eq!(st.numbers.agents, h);
        // chat 1 h + own timer: the timer ran until the real now, clipped at `now`
        assert_eq!(st.numbers.you, h + h / 2);
        assert_eq!(st.numbers.tokens, 300, "a third of the item's time is today");
        let stats = st.goals.iter().find(|g| g.id == "stats").unwrap();
        assert_eq!((stats.numbers.you, stats.numbers.agents), (h, h));
        let other = st.goals.iter().find(|g| g.id.is_empty()).unwrap();
        assert_eq!(other.numbers.you, h / 2);
        assert_eq!(st.buckets.len(), 18, "hours of today up to 18:00");
        // the week holds yesterday too
        let week = of_store(&s, &[], "jul", Period::Week, now);
        assert_eq!(week.numbers.agents, 3 * h);
        assert_eq!(week.numbers.tokens, 900);
    }

    #[test]
    fn periods_start_at_midnight_monday_first_and_january() {
        let now = Local.with_ymd_and_hms(2026, 11, 15, 18, 0, 0).unwrap().timestamp(); // a Sunday
        let at = |y, m, d| Local.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap().timestamp();
        assert_eq!(Period::Day.range(now).0, at(2026, 11, 15));
        let (from, days) = Period::Week.range(now);
        assert_eq!((from, days.len()), (at(2026, 11, 9), 7));
        assert_eq!(Period::Month.range(now).1.len(), 15);
        assert_eq!(Period::Days7.range(now), (at(2026, 11, 9), Period::Week.range(now).1), "a Sunday: the same days");
        assert_eq!(Period::Year.range(now).1.len(), 11);
    }
}
