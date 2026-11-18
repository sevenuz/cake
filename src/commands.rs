use crate::config::Config;
use crate::item::{Item, Status};
use crate::store::Store;
use crate::view;
use crate::Selector;
use crate::{error, git, out, say, style, util};
use std::error::Error;
use std::collections::HashSet;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::theme;
use ratatui::style::{Modifier, Style};

/// fields of a new item besides selector and content
pub struct AddOptions {
    /// None: a note
    pub status: Option<Status>,
    pub assignee: Option<String>,
    pub owner: Option<String>,
    /// the human, default owner of tasks
    pub user: String,
    pub due: Option<String>,
    pub deadline: Option<String>,
    pub priority: Option<u64>,
    pub estimate: Option<String>,
    pub blocked_by: Vec<String>,
}

/// adds an item; without content the editor opens, which needs a terminal
/// what a command reports, printed as json with `--json`
pub enum Report {
    None,
    /// items after the command, by id
    Items(Vec<String>),
    /// like Items, without body
    Brief(Vec<String>),
    /// ids of removed items
    Removed(Vec<String>),
    Value(serde_json::Value),
}

fn store_report(root: &Path) -> Report {
    Report::Value(serde_json::json!({ "store": root }))
}

pub fn add<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
    content: Option<String>,
    options: AddOptions,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("add {:?}, content: {:?}", selector, content));
    let content = content.ok_or_else(|| error::invalid("No content, use -m TEXT (- for stdin)."))?;
    let id = match selector.ids.first() {
        Some(id) => id.to_string(),
        None => util::free_id(|id| store.get_item(id).is_some() || store.is_archived(id) || id == crate::store::inner::ARCHIVE),
    };
    util::check_id(&id)?;
    let mut item = Item::new(
        id.clone(),
        selector.children,
        selector.parents.first().cloned(),
        selector.tags,
    );
    store.check_existence(&item, false)?;
    if let Some(missing) = options.blocked_by.iter().find(|i| store.get_item(i).is_none())
    {
        return Err(error::not_found(missing));
    }
    if let Some(st) = options.status {
        item.set_status(st)?;
    }
    item.set_assignee(options.assignee);
    // subtasks are reviewed by whoever owns the parent
    let parent_owner = item.parents().first().and_then(|p| store.get_item(p)).and_then(|p| p.owner().cloned());
    let owner = options.owner.or(parent_owner).or_else(|| (options.status == Some(Status::Task)).then(|| options.user.clone()));
    item.set_owner(owner);
    item.set_due(options.due).map_err(error::invalid)?;
    item.set_deadline(options.deadline).map_err(error::invalid)?;
    item.set_priority(options.priority).map_err(error::invalid)?;
    item.set_estimate(options.estimate).map_err(error::invalid)?;
    item.set_blocked_by(options.blocked_by);
    item.set_by(config.author(store.root()));
    item.set_content(content);
    debug(&item.content());
    store.add(item)?;
    view::print(&config, store.get_item(&id).unwrap().print_long())?;
    Ok(Report::Items(vec![id]))
}

/// changes without an editor; `priority` 0 and `due` "-" clear the field
pub struct EditOptions {
    /// (item as markdown before, after) from `edit_in_editor`
    pub edited: Option<(String, String)>,
    pub body: Option<String>,
    pub priority: Option<u64>,
    pub due: Option<String>,
    pub deadline: Option<String>,
    /// "-" clears
    pub estimate: Option<String>,
    /// Some(None) clears
    pub assignee: Option<Option<String>>,
    pub owner: Option<Option<String>>,
}

/// opens the item file (frontmatter and body) in the editor, without the
/// lock, so other cake commands keep working meanwhile
pub fn edit_in_editor(config: &Config, root: &Path, id: &str) -> Result<(String, String), Box<dyn Error>> {
    let store = Store::new_except(root, &[])?;
    let id = &store.resolve(id)?;
    let text = store.get_item(id).ok_or_else(|| error::not_found(id))?.to_markdown();
    if !std::io::stdin().is_terminal() || out::json() {
        return Err(error::invalid("cake edit without -m/--prio/--due needs a terminal and no --json."));
    }
    let edited = util::input_from_external_editor(&config.editor, Some(&text))?;
    Ok((text, edited))
}

/// applies `options`; an editor result is refused if the item changed meanwhile
pub fn edit<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    id: &str,
    options: EditOptions,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("edit {:?}", id));
    let old = store.get_item(id).ok_or_else(|| error::not_found(id))?;
    if options.body.is_some()
        || options.priority.is_some()
        || options.due.is_some()
        || options.deadline.is_some()
        || options.estimate.is_some()
        || options.assignee.is_some()
        || options.owner.is_some()
    {
        let mut item = old.clone();
        if let Some(body) = options.body {
            item.set_content(body);
        }
        if let Some(p) = options.priority {
            item.set_priority(Some(p).filter(|p| *p != 0)).map_err(error::invalid)?;
        }
        if let Some(d) = options.due {
            item.set_due(Some(d).filter(|d| d != "-")).map_err(error::invalid)?;
        }
        if let Some(d) = options.deadline {
            item.set_deadline(Some(d).filter(|d| d != "-")).map_err(error::invalid)?;
        }
        if let Some(e) = options.estimate {
            item.set_estimate(Some(e).filter(|e| e != "-")).map_err(error::invalid)?;
        }
        if let Some(a) = options.assignee {
            item.set_assignee(a);
        }
        if let Some(o) = options.owner {
            item.set_owner(o);
        }
        store.get_item_mut(id).unwrap().set(item);
        view::print(&config, store.get_item(id).unwrap().print_long())?;
        return Ok(Report::Items(vec![id.to_owned()]));
    }
    let (text, edited) = options.edited.ok_or_else(|| error::invalid("Nothing to edit."))?;
    if old.to_markdown() != text {
        return Err(error::conflict(format!(
            "{} changed while you were editing it, nothing written. Your version: {}",
            id,
            std::env::temp_dir().join("editable.md").display()
        )));
    }
    if edited == text {
        say!("{} unchanged.", id);
        return Ok(Report::Items(vec![id.to_owned()]));
    }
    let item = Item::from_markdown(&edited, id, old.timestamp())?;
    if item.id() != id {
        return Err(error::invalid("Changing the id is not supported."));
    }
    store.check_existence(&item, true)?;
    store.edit(item)?;
    view::print(&config, store.get_item(id).unwrap().print_long())?;
    Ok(Report::Items(vec![id.to_owned()]))
}

/// ids for commands that change items: refuses an empty selector (it would
/// match everything) and unknown ids
fn select(store: &Store, selector: &Selector) -> Result<Vec<String>, Box<dyn Error>> {
    if selector.is_empty() {
        return Err(error::invalid("No items selected, give ids or selectors."));
    }
    if let Some(id) = selector.ids.iter().find(|id| store.get_item(id).is_none()) {
        return Err(error::not_found(id));
    }
    Ok(selector.get(store, true))
}

pub fn remove<F>(debug: F, store: &mut Store, selector: Selector) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("remove {:?}", selector));
    let ids = select(store, &selector)?;
    for id in &ids {
        store.remove(id)?;
    }
    say!("{} removed.", ids.len());
    Ok(Report::Removed(ids))
}


pub fn start<F>(debug: F, store: &mut Store, selector: Selector) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("start {:?}", selector));
    let ids = select(store, &selector)?;
    for id in &ids {
        store
            .get_item_mut(id)
            .expect("Could not found id")
            .start()?;
    }
    say!("{} started.", ids.len());
    Ok(Report::Items(ids))
}

/// stops the selected items, without selector the running items you own
pub fn stop<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("stop {:?}", selector));
    let ids = if selector.is_empty() {
        let me = config.author(store.root());
        let mut ids: Vec<String> = store
            .get()
            .values()
            .filter(|i| i.is_started() && i.by() == Some(&me))
            .map(|i| i.id().to_owned())
            .collect();
        ids.sort();
        ids
    } else {
        select(store, &selector)?
    };
    for id in &ids {
        store.get_item_mut(id).expect("Could not found id").stop()?;
    }
    say!("{} stopped.", ids.len());
    Ok(Report::Items(ids))
}

/// re-parents an item, "-" detaches it
pub fn mv<F>(debug: F, store: &mut Store, id: &str, parent: &str) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("mv {:?} {:?}", id, parent));
    let parent = if parent == "-" { None } else { Some(parent) };
    store.move_item(id, parent)?;
    match parent {
        Some(p) => say!("{} moved under {}.", id, p),
        None => say!("{} detached.", id),
    }
    Ok(Report::Items(vec![id.to_owned()]))
}

pub fn rename<F>(debug: F, store: &mut Store, id: &str, new: &str) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("rename {:?} {:?}", id, new));
    store.rename(id, new)?;
    say!("{} is now {}.", id, new);
    Ok(Report::Items(vec![new.to_owned()]))
}

/// moves closed subtrees closed for at least `older` into items/archive/
/// (with `dry_run` only lists them)
pub fn archive<F>(debug: F, store: &mut Store, older: &str, dry_run: bool) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("archive --older {:?} dry_run {}", older, dry_run));
    let (age, now) = (util::parse_age(older)?, util::timestamp());
    if dry_run {
        let ids = store.archive_candidates(age, now);
        if !out::json() {
            let short = store.short_ids();
            for id in &ids {
                println!("{}", line(store.get_item(id).unwrap(), &short[id], style::color()));
            }
        }
        say!("{} would be archived (closed longer than {}).", ids.len(), older);
        return Ok(Report::Value(serde_json::json!({ "archived": ids, "dry_run": true })));
    }
    let ids = store.archive(age, now);
    say!("{} archived (closed longer than {}).", ids.len(), older);
    Ok(Report::Value(serde_json::json!({ "archived": ids })))
}

/// moves `id` and its subtree to the store at `target`, top-level there;
/// the target is locked and committed on its own
pub fn mv_to_store<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    id: &str,
    target: &Path,
    message: &str,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("mv {:?} --to-store {:?}", id, target));
    store.get_item(id).ok_or_else(|| error::not_found(id))?;
    if !target.join("items").is_dir() {
        return Err(error::invalid(format!("{} is not a store.", target.display())));
    }
    if fs::canonicalize(target)? == fs::canonicalize(store.root())? {
        return Err(error::invalid("That is this store."));
    }
    let mut ids = vec![id.to_owned()];
    let mut n = 0;
    while n < ids.len() {
        let children = store.get_item(&ids[n]).unwrap().children().clone();
        for c in children {
            if !ids.contains(&c) {
                ids.push(c);
            }
        }
        n += 1;
    }
    let mut lock = util::lock_file(target)?;
    let _guard = match lock.as_mut() {
        Some(l) => Some(util::lock(l, true)?),
        None => None,
    };
    let mut other = Store::new_except(target, &[])?;
    if let Some(taken) = ids.iter().find(|i| other.get_item(i).is_some()) {
        return Err(error::invalid(format!("{} already exists in {}.", taken, target.display())));
    }
    for i in &ids {
        let mut item = store.get_item(i).unwrap().clone();
        if i == id {
            item.set_parent(None);
        }
        other.add(item)?;
    }
    if let Some(session) = config.session() {
        other.add_session(&session);
    }
    let changed = other.save()?;
    if !config.disable_git && git::is_store_repo(target) {
        git::commit(target, &changed, message)?;
    }
    for i in ids.iter().rev() {
        store.remove(i)?;
    }
    say!("{} moved to {} ({} items).", id, target.display(), ids.len());
    Ok(Report::Value(serde_json::json!({ "moved": ids, "store": target })))
}


pub fn set_status<F>(
    debug: F,
    store: &mut Store,
    selector: Selector,
    status: Status,
    note: Option<String>,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("set_status {:?} {:?}", selector, status));
    let ids = select(store, &selector)?;
    for id in &ids {
        let item = store.get_item_mut(id).unwrap();
        item.set_status(status)?;
        if let Some(n) = &note {
            item.append_log(n);
        }
    }
    say!("{} {}.", ids.len(), status.as_str());
    Ok(Report::Items(ids))
}

/// Some(message) while `max_review` (store config, -1 off) items wait in review
pub fn reviews_full(store: &Store, sc: &crate::config::StoreConfig) -> Option<String> {
    let waiting = store.get().values().filter(|i| i.status() == Some(Status::Review)).count() as i64;
    (sc.max_review >= 0 && waiting >= sc.max_review)
        .then(|| format!("waiting for review: {} items, max_review is {}; no new work for agents", waiting, sc.max_review))
}

/// tasks without owner get `user` (whoever made them tasks)
pub fn default_owner(store: &mut Store, report: &Report, user: &str) {
    if let Report::Items(ids) = report {
        for id in ids {
            let item = store.get_item_mut(id).unwrap();
            if item.owner().is_none() {
                item.set_owner(Some(user.to_string()));
            }
        }
    }
}

/// selected items become plain notes
pub fn to_note(store: &mut Store, selector: Selector) -> Result<Report, Box<dyn Error>> {
    let ids = select(store, &selector)?;
    for id in &ids {
        store.get_item_mut(id).unwrap().set_note()?;
    }
    say!("{} note.", ids.len());
    Ok(Report::Items(ids))
}


/// logs "comment (user): text"; work in review or doing goes back to its
/// assignee as a task
pub fn comment(store: &mut Store, id: &str, user: &str, text: &str) -> Result<Report, Box<dyn Error>> {
    let item = store.get_item_mut(id).ok_or_else(|| error::not_found(id))?;
    item.append_log(&format!("comment ({}): {}", user, text.trim()));
    if matches!(item.status(), Some(Status::Review) | Some(Status::Doing)) {
        item.set_status(Status::Task)?;
    }
    say!("{} commented.", id);
    Ok(Report::Items(vec![id.to_owned()]))
}

/// one line for prompts, tmux and status lines, segments from config `status`
/// "1h05", "0h00"
fn hm(s: i64) -> String {
    format!("{}h{:02}", s / 3600, s / 60 % 60)
}

/// a sparkline of `values`, scaled to the largest
pub(crate) fn spark(values: &[i64]) -> String {
    let marks: Vec<char> = if style::ascii() { " .:-=+*#".chars().collect() } else { " ▁▂▃▄▅▆▇█".chars().collect() };
    let max = values.iter().copied().max().unwrap_or(0).max(1);
    values.iter().map(|v| if *v <= 0 { marks[0] } else { marks[1 + (*v * (marks.len() as i64 - 2) / max) as usize] }).collect()
}

/// numbers for a period: this store, or all known stores with `all`
pub fn stats(config: &Config, store: &Store, period: &str, all: bool) -> Result<Report, Box<dyn Error>> {
    use crate::stats::{self, Period};
    let period = Period::parse(period).ok_or_else(|| error::invalid(format!("'{}' is no period: day, 7d, week, month, year", period)))?;
    let now = util::timestamp();
    let user = config.user(store.root());
    let (from, _) = period.range(now);
    let same = |a: &Path, b: &Path| fs::canonicalize(a).ok() == fs::canonicalize(b).ok();
    let known = crate::stores::all().unwrap_or_default();
    let here = known.iter().find(|(_, p)| same(p, store.root())).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut rows: Vec<(String, stats::StoreStats)> = vec![(here, stats::of_store(store, &crate::log::read(store.root(), from, now), &user, period, now))];
    if all {
        for (name, path) in known.iter().filter(|(_, p)| !same(p, store.root())) {
            let s = Store::new_except(path, &[])?;
            rows.push((name.clone(), stats::of_store(&s, &crate::log::read(path, from, now), &user, period, now)));
        }
    }
    if !out::json() {
        let w = rows.iter().map(|(n, _)| n.chars().count()).max().unwrap_or(5).max(5);
        println!("{} {}", period.name(), util::format_timestamp(from)[..10].to_string() + " – now");
        let cols = stats::shown();
        let head: String = cols.iter().map(|c| format!("  {:>7}", cut_to(c, 7))).collect();
        println!("{:w$}  {:>6}  {:>6}{}  {:>4}  {}", "", "you", "agents", head, "done", period.bucket(), w = w);
        let line = |name: &str, n: &stats::Numbers, buckets: &str| {
            let sums: String = cols.iter().map(|c| format!("  {:>7}", stats::human(n.sum(c)))).collect();
            println!("{:w$}  {:>6}  {:>6}{}  {:>4}  {}", name, hm(n.you), hm(n.agents), sums, n.done, buckets, w = w)
        };
        for (name, s) in &rows {
            line(name, &s.numbers, &spark(&s.buckets.iter().map(|(_, v)| *v).collect::<Vec<_>>()));
        }
        if rows.len() > 1 {
            line("all", &stats::sum(rows.iter().map(|(_, s)| s)), "");
        }
        let s = &rows[0].1;
        println!("overdue {} · review {} · questions {} · streak {} d", s.overdue, s.review, s.questions, s.streak);
        let goals: Vec<String> = s.goals.iter().map(|g| format!("{} {}", if g.id.is_empty() { "other" } else { &g.id }, hm(g.numbers.you + g.numbers.agents))).collect();
        if !goals.is_empty() {
            println!("goals ({}): {}", rows[0].0, goals.join(" · "));
        }
    }
    let stores: Vec<serde_json::Value> =
        rows.iter().map(|(n, s)| serde_json::json!({ "store": n, "stats": s })).collect();
    Ok(Report::Value(serde_json::json!({ "period": period.name(), "stores": stores })))
}

pub fn status(config: &Config, store: &Store) -> Result<Report, Box<dyn Error>> {
    let me = config.user(store.root());
    let short = store.short_ids();
    let items: Vec<&Item> = store.get().values().collect();
    let mut mine: Vec<(&Item, i64)> = items
        .iter()
        .filter(|i| i.by() == Some(&me))
        .filter_map(|i| i.running_seconds().map(|s| (*i, s)))
        .collect();
    mine.sort_by_key(|(i, _)| i.id().to_owned());
    let others = items.iter().filter(|i| i.is_started() && i.by() != Some(&me)).count();
    let ready = items.iter().filter(|i| store.is_ready(i.id())).count();
    let midnight = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(chrono::Local).earliest())
        .map_or(0, |t| t.timestamp());
    // everyone's time on this store, e.g. you and your agents on one project
    let today = union_seconds(
        items
            .iter()
            .flat_map(|i| i.intervals())
            .map(|(s, e)| (s.max(midnight), e))
            .filter(|(s, e)| e > s)
            .collect(),
    );
    let hm = |s: i64| format!("{}h{:02}", s / 3600, s / 60 % 60);
    // week and streak need the stats of the week (session log too): only
    // computed when one of them is shown
    let week = std::cell::OnceCell::new();
    let week = || {
        week.get_or_init(|| {
            let now = util::timestamp();
            let (from, _) = crate::stats::Period::Week.range(now);
            let log = crate::log::read(store.root(), from.min(now - 400 * 86400), now);
            crate::stats::of_store(store, &log, &me, crate::stats::Period::Week, now)
        })
    };
    let mut tags = serde_json::Map::new();
    let mut parts: Vec<String> = vec![];
    for segment in &config.status {
        match segment.as_str() {
            "mine" => parts.extend(mine.iter().map(|(i, s)| {
                let mut title: String = i.title().chars().take(24).collect();
                if title.len() < i.title().len() {
                    title.push_str(style::g().more);
                }
                format!("{} {} {} {}", style::g().running, short[i.id()], title, hm(*s))
            })),
            "others" if others > 0 => parts.push(format!("{} {} others", style::g().running, others)),
            "ready" if ready > 0 => parts.push(format!("ready {}", ready)),
            "review" => {
                let n = items.iter().filter(|i| i.status() == Some(Status::Review)).count();
                tags.insert("review".into(), n.into());
                if n > 0 {
                    parts.push(format!("review {}", n));
                }
            }
            "today" if today >= 60 => parts.push(format!("today {}", hm(today))),
            "week" => {
                let n = &week().numbers;
                if n.you + n.agents >= 60 {
                    parts.push(format!("week {}", hm(n.you + n.agents)));
                }
            }
            "streak" if week().streak > 1 => parts.push(format!("streak {} d", week().streak)),
            s if s.starts_with("tag:") => {
                let tag = &s[4..];
                let n = items.iter().filter(|i| i.is_open() && i.tags().iter().any(|t| t == tag)).count();
                tags.insert(tag.to_string(), n.into());
                if n > 0 {
                    parts.push(format!("{} {}", tag, n));
                }
            }
            _ => (),
        }
    }
    if !parts.is_empty() {
        say!("{}", parts.join(style::g().sep));
    }
    let mine: Vec<_> = mine
        .iter()
        .map(|(i, s)| serde_json::json!({ "id": i.id(), "title": i.title(), "seconds": s }))
        .collect();
    Ok(Report::Value(serde_json::json!({
        "user": me, "mine": mine, "others": others, "ready": ready, "today_seconds": today, "tags": tags,
    })))
}

/// total length of possibly overlapping (start, end) intervals, overlaps once
fn union_seconds(mut spans: Vec<(i64, i64)>) -> i64 {
    spans.sort();
    let mut total = 0;
    let mut current: Option<(i64, i64)> = None;
    for (s, e) in spans {
        current = match current {
            Some((cs, ce)) if s <= ce => Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                total += ce - cs;
                Some((s, e))
            }
            None => Some((s, e)),
        };
    }
    total + current.map_or(0, |(s, e)| e - s)
}

#[cfg(test)]
mod tests {
    #[test]
    fn union_counts_overlaps_once() {
        assert_eq!(super::union_seconds(vec![(0, 10), (5, 15), (20, 30)]), 25);
        assert_eq!(super::union_seconds(vec![(20, 30), (0, 40)]), 40);
        assert_eq!(super::union_seconds(vec![]), 0);
    }
}

pub fn list<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
    long: bool,
    all: bool,
    scope: Option<&str>,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("list {:?} long: {:?} scope: {:?}", selector, long, scope));
    let show_closed = all || !selector.ids.is_empty() || !selector.status.is_empty();
    let in_scope = |id: &str| scope.is_none_or(|g| store.has_ancestor_or_is(id, g));
    let ids: HashSet<String> = selector
        .get(store, true)
        .into_iter()
        .filter(|id| show_closed || store.get_item(id).map_or(false, |i| i.status().map_or(true, |s| s.is_open())))
        .filter(|id| in_scope(id))
        .collect();
    let order = tree(store, &ids);

    if long {
        let text = order
            .iter()
            .map(|(_, id)| store.get_item(id).unwrap().to_string())
            .collect::<Vec<_>>()
            .join("\n---\n");
        view::print(&config, text)?;
    } else if !out::json() {
        let color = style::color();
        let short = store.short_ids();
        for (prefix, id) in &order {
            println!("{}{}", prefix, line(store.get_item(id).unwrap(), &short[id], color));
        }
    }
    let ids = order.into_iter().map(|(_, id)| id).collect();
    Ok(if long { Report::Items(ids) } else { Report::Brief(ids) })
}

/// depth-first order of `ids` as (tree prefix, id): items whose parent is not
/// listed are roots; siblings by score, then creation
fn tree(store: &Store, ids: &HashSet<String>) -> Vec<(String, String)> {
    let now = util::timestamp();
    let key = |id: &String| {
        let i = store.get_item(id).unwrap();
        (std::cmp::Reverse(store.score(id, now)), i.timestamp(), id.clone())
    };
    let sorted = |mut v: Vec<String>| {
        v.sort_by_cached_key(|id| key(id));
        v
    };
    fn walk(
        store: &Store,
        ids: &HashSet<String>,
        sorted: &dyn Fn(Vec<String>) -> Vec<String>,
        id: &str,
        prefix: (&str, &str),
        out: &mut Vec<(String, String)>,
    ) {
        if out.iter().any(|(_, seen)| seen == id) {
            return; // cycle
        }
        out.push((prefix.0.to_string(), id.to_string()));
        let children = sorted(
            store.get_item(id).unwrap().children().iter().filter(|c| ids.contains(*c)).cloned().collect(),
        );
        for (n, c) in children.iter().enumerate() {
            let last = n + 1 == children.len();
            let g = style::g();
            let own = format!("{}{}", prefix.1, if last { g.last } else { g.branch });
            let below = format!("{}{}", prefix.1, if last { g.space } else { g.pipe });
            walk(store, ids, sorted, c, (&own, &below), out);
        }
    }
    let roots = sorted(
        ids.iter()
            .filter(|id| store.get_item(id).unwrap().parents().first().map_or(true, |p| !ids.contains(p)))
            .cloned()
            .collect(),
    );
    let mut out = vec![];
    for r in roots {
        walk(store, ids, &sorted, &r, ("", ""), &mut out);
    }
    out
}

/// one list line: status marker, short id, title, tags
fn line(item: &Item, short: &str, color: bool) -> String {
    let g = style::g();
    let marker = match item.status() {
        None => g.note,
        Some(Status::Inbox) => g.inbox,
        Some(Status::Task) => g.todo,
        Some(Status::Review) => g.review,
        Some(Status::Doing) if item.is_started() => g.running,
        Some(Status::Doing) => g.doing,
        Some(Status::Done) => g.done,
        Some(Status::Dropped) => g.dropped,
    };
    let tags: String = item.tags().iter().map(|t| format!(" #{}", t)).collect();
    let open = item.status().map_or(true, |s| s.is_open());
    if !color {
        return format!("{} {} {}{}", marker, short, item.title(), tags);
    }
    let t = theme::t();
    let marker = match item.status() {
        Some(Status::Doing) if item.is_started() => {
            theme::paint(marker, Style::default().fg(t.running).add_modifier(Modifier::BOLD))
        }
        Some(Status::Doing) => theme::fg(marker, t.doing),
        _ => marker.to_string(),
    };
    let title = if open { item.title() } else { theme::fg(&item.title(), t.dim) };
    let text = format!("{} {}", theme::fg(short, t.dim), title);
    format!("{} {}{}", marker, text, theme::fg(&tags, t.dim))
}

/// shows an item by id, or any markdown file by path
pub fn show<F>(debug: F, config: &Config, store: &Store, target: &str) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("show: {:?}", target));
    let data = match store.get_item(target) {
        Some(item) => {
            view::print(&config, item.print_long())?;
            let ids: HashSet<String> = item.children().iter().cloned().collect();
            let children = tree(store, &ids);
            if !children.is_empty() && !out::json() {
                let color = style::color();
                print!("{}", if item.content().ends_with('\n') { "\n" } else { "\n\n" });
                let short = store.short_ids();
                for (prefix, id) in children {
                    println!("{}{}", prefix, line(store.get_item(&id).unwrap(), &short[&id], color));
                }
            }
            return Ok(Report::Items(vec![target.to_owned()]));
        }
        None if Path::new(target).is_file() => fs::read_to_string(target)?,
        None => return Err(error::coded("not_found", format!("{} is no item id or file.", target))),
    };
    view::print(&config, data.clone())?;
    Ok(Report::Value(serde_json::json!({ "path": target, "content": data })))
}

/// Creates the store: inside a git repo a worktree of `config.git_branch` at
/// `<repo>/.cake` (attached if the branch exists locally or on the remote),
/// otherwise a plain `.cake` dir in the current directory.
/// With `global`, the global store becomes a git repo on `main`.
pub fn init<F>(debug: F, config: &Config, global: bool, store: Option<&Path>, parent: Option<&str>, items: &Store) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    if parent.is_some() && (store.is_none() || global) {
        return Err(error::invalid("--parent links a dir to a store: use it with --store DIR."));
    }
    // a separate store, used from the current dir (e.g. a paper repo)
    if let Some(p) = store.filter(|_| !global) {
        if p.join("items").is_dir() {
            // an existing store: link this dir to it
            let cwd = std::env::current_dir()?;
            let abs = fs::canonicalize(p)?;
            if cwd.starts_with(&abs) || abs.parent().is_some_and(|r| cwd.starts_with(r) && abs.ends_with(util::STORE_DIR)) {
                return Err(error::invalid(format!("There is already a store at {}.", p.display())));
            }
            if let Some(id) = parent.filter(|id| items.get_item(id).is_none()) {
                return Err(error::not_found(id));
            }
            let name = abs.parent().filter(|_| abs.ends_with(util::STORE_DIR)).unwrap_or(&abs).file_name().map_or("store".into(), |n| n.to_string_lossy().to_string());
            crate::stores::link(&abs, &name, &cwd, parent)?;
            say!("{} uses the store {}{}.", cwd.display(), p.display(), parent.map_or(String::new(), |g| format!(", captures go under {}", g)));
            return Ok(store_report(p));
        }
        if parent.is_some() {
            return Err(error::invalid("--parent needs an existing store with that item; create the store first."));
        }
        let files = create_store_files(p)?;
        if !config.disable_git {
            git::init_repo(p)?;
            git::commit(p, &files, "cake: init")?;
        }
        let cwd = std::env::current_dir()?;
        let abs = fs::canonicalize(p)?;
        let dirs = if cwd.starts_with(&abs) { vec![] } else { vec![cwd.clone()] };
        let name = cwd.file_name().map_or("store".into(), |n| n.to_string_lossy().to_string());
        crate::stores::register(&abs, &name, &dirs)?;
        say!("New store {} created, used from {} :)", p.display(), cwd.display());
        return Ok(store_report(p));
    }
    if global {
        let p = util::global_store()?;
        debug(&format!("init: global store {:?}", p));
        let exists = if config.disable_git {
            p.join("items").is_dir()
        } else {
            git::is_store_repo(&p)
        };
        if exists {
            return Err(error::invalid(format!("There is already a store at {}.", p.display())));
        }
        // an existing plain global store is adopted with its items
        let mut files = create_store_files(&p)?;
        if fs::read_dir(p.join("items"))?.next().is_some() {
            files.push(p.join("items"));
        }
        if !config.disable_git {
            git::init_repo(&p)?;
            git::commit(&p, &files, "cake: init")?;
        }
        say!("New global store {} created :)", p.display());
        return Ok(store_report(&p));
    }
    let cwd = std::env::current_dir()?;
    if config.disable_git || !git::is_repo(&cwd) {
        let p = cwd.join(util::STORE_DIR);
        debug(&format!("init: new store {:?}", p));
        if p.join("items").is_dir() {
            return Err(error::invalid("There is already a store here."));
        }
        create_store_files(&p)?;
        say!("New store {} created :)", p.display());
        return Ok(store_report(&p));
    }
    let repo = git::toplevel(&cwd)?;
    let p = repo.join(util::STORE_DIR);
    if p.exists() {
        return Err(error::invalid(format!("{} already exists.", p.display())));
    }
    debug(&format!("init: worktree {:?} on {}", p, config.git_branch));
    let attached = git::add_worktree(&repo, &p, &config.git_branch, &config.git_remote_name)?;
    git::exclude(&repo, &format!("/{}/", util::STORE_DIR))?;
    if attached {
        fs::create_dir_all(p.join("items"))?;
        say!("Attached store {} on branch {} :)", p.display(), config.git_branch);
    } else {
        let files = create_store_files(&p)?;
        git::commit(&p, &files, "cake: init")?;
        say!("New store {} created on branch {} :)", p.display(), config.git_branch);
    }
    Ok(store_report(&p))
}

/// creates `items/` and `.gitignore` in `root`, returns the files to commit
fn create_store_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    fs::create_dir_all(root.join("items"))?;
    let gitignore = root.join(".gitignore");
    fs::write(&gitignore, util::STORE_GITIGNORE)?;
    Ok(vec![gitignore])
}

pub fn sync<F>(
    debug: F,
    config: &Config,
    root: &Path,
    publish: &Option<String>,
    cont: bool,
    abort: bool,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    if config.disable_git || !git::is_store_repo(root) {
        return Err(error::invalid(format!("{} is not a git store, see `cake help init`.", root.display())));
    }
    if abort {
        debug("sync: abort");
        git::sync_abort(root)?;
        say!("Merge aborted.");
        return Ok(Report::Value(serde_json::json!({ "aborted": true })));
    } else if cont {
        debug("sync: continue");
        git::sync_continue(root)?;
        say!("Synced.");
    } else if let Some(remote) = publish {
        let remote = if remote.is_empty() { &config.git_remote_name } else { remote };
        debug(&format!("sync: publish to {}", remote));
        git::publish(root, remote)?;
        say!("Published to {}.", remote);
        return Ok(Report::Value(serde_json::json!({ "published": remote })));
    } else {
        debug("sync");
        git::sync(root)?;
        say!("Synced.");
    }
    Ok(Report::Value(serde_json::json!({ "synced": true })))
}

pub fn config<F>(debug: F, config: &Config) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    if out::json() {
        return Err(error::invalid("cake config opens an editor, it has no --json output."));
    }
    debug(&format!(
        "config: open editor {:?} with file {:?}",
        config.editor,
        util::config_file()?
    ));
    Command::new(&config.editor)
        .arg(util::config_file()?)
        .status()?;
    Ok(Report::None)
}

/// at most `n` characters of `s`
fn cut_to(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
