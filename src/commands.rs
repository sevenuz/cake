use crate::config::Config;
use crate::item::{Item, Status};
use crate::store::Store;
use crate::view;
use crate::Selector;
use crate::{error, git, out, say, util};
use std::error::Error;
use std::collections::HashSet;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;
use termimad::crossterm::style::Stylize;

/// fields of a new item besides selector and content
pub struct AddOptions {
    pub todo: bool,
    pub due: Option<String>,
    pub priority: Option<u64>,
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
    let id = if selector.ids.is_empty() {
        util::generate_id()
    } else {
        selector.ids.first().unwrap().to_string()
    };
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
    if options.todo {
        item.set_status(Status::Todo)?;
    }
    item.set_due(options.due).map_err(error::invalid)?;
    item.set_priority(options.priority).map_err(error::invalid)?;
    item.set_blocked_by(options.blocked_by);
    item.set_by(config.author(store.root()));
    item.set_content(match content {
        Some(c) => c,
        None => return Err(error::invalid("No content, use -m TEXT (- for stdin).")),
    });
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
    if options.body.is_some() || options.priority.is_some() || options.due.is_some() {
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

pub fn tag<F>(
    debug: F,
    store: &mut Store,
    selector: Selector,
    tags: &Option<String>,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("tag {:?} new_tags {:?}", selector, tags));
    let ids = select(store, &selector)?;
    for id in &ids {
        store
            .get_item_mut(id)
            .unwrap()
            .append_tags(util::split_comma_tags(
                tags.to_owned().unwrap_or("".to_string()),
            ))
            .remove_tags(util::split_comma_exclude_tags(
                tags.to_owned().unwrap_or("".to_string()),
            ));
    }
    say!("{} tagged.", ids.len());
    Ok(Report::Items(ids))
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

/// adds or removes blockers of `id`
pub fn block<F>(
    debug: F,
    store: &mut Store,
    id: &str,
    by: &str,
    remove: bool,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("block {:?} by {:?}, remove: {:?}", id, by, remove));
    let blockers = util::split_comma_cleanup(by.to_owned());
    if remove {
        store.unblock(id, &blockers)?;
    } else {
        store.block(id, &blockers)?;
    }
    let left = store.get_item(id).unwrap().blocked_by();
    say!("{} blocked by: {}", id, util::vec_to_str(left));
    Ok(Report::Items(vec![id.to_owned()]))
}

pub fn set_status<F>(
    debug: F,
    store: &mut Store,
    selector: Selector,
    status: Status,
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("set_status {:?} {:?}", selector, status));
    let ids = select(store, &selector)?;
    for id in &ids {
        store.get_item_mut(id).unwrap().set_status(status)?;
    }
    say!("{} {}.", ids.len(), status.as_str());
    Ok(Report::Items(ids))
}

/// one line for prompts, tmux and status lines, segments from config `status`
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
    let mut tags = serde_json::Map::new();
    let mut parts: Vec<String> = vec![];
    for segment in &config.status {
        match segment.as_str() {
            "mine" => parts.extend(mine.iter().map(|(i, s)| {
                let mut title: String = i.title().chars().take(24).collect();
                if title.len() < i.title().len() {
                    title.push('…');
                }
                format!("▶ {} {} {}", short[i.id()], title, hm(*s))
            })),
            "others" if others > 0 => parts.push(format!("▶ {} others", others)),
            "ready" if ready > 0 => parts.push(format!("ready {}", ready)),
            "today" if today > 0 => parts.push(format!("today {}", hm(today))),
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
        say!("{}", parts.join(" · "));
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
) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("list {:?} long: {:?}", selector, long));
    let show_closed = all || !selector.ids.is_empty() || !selector.status.is_empty();
    let ids: HashSet<String> = selector
        .get(store, true)
        .into_iter()
        .filter(|id| show_closed || store.get_item(id).map_or(false, |i| i.status().map_or(true, |s| s.is_open())))
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
        let color = std::io::stdout().is_terminal();
        let short = store.short_ids();
        for (prefix, id) in &order {
            println!("{}{}", prefix, line(store.get_item(id).unwrap(), &short[id], color));
        }
    }
    let ids = order.into_iter().map(|(_, id)| id).collect();
    Ok(if long { Report::Items(ids) } else { Report::Brief(ids) })
}

/// depth-first order of `ids` as (tree prefix, id): items whose parent is not
/// listed are roots; siblings by priority, due, then creation
fn tree(store: &Store, ids: &HashSet<String>) -> Vec<(String, String)> {
    let key = |id: &String| {
        let i = store.get_item(id).unwrap();
        (i.priority().unwrap_or(u64::MAX), i.due().is_none(), i.due().cloned(), i.timestamp(), id.clone())
    };
    let sorted = |mut v: Vec<String>| {
        v.sort_by_key(|id| key(id));
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
            let own = format!("{}{}", prefix.1, if last { "└─ " } else { "├─ " });
            let below = format!("{}{}", prefix.1, if last { "   " } else { "│  " });
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
    let marker = match item.status() {
        None => "·",
        Some(Status::Todo) => "○",
        Some(Status::Doing) if item.is_started() => "▶",
        Some(Status::Doing) => "◐",
        Some(Status::Done) => "✓",
        Some(Status::Dropped) => "✗",
    };
    let tags: String = item.tags().iter().map(|t| format!(" #{}", t)).collect();
    let open = item.status().map_or(true, |s| s.is_open());
    if !color {
        return format!("{} {} {}{}", marker, short, item.title(), tags);
    }
    let marker = match marker {
        "▶" => marker.green().bold(),
        "◐" => marker.yellow(),
        _ => marker.stylize(),
    };
    let title = if open { item.title().stylize() } else { item.title().dark_grey() };
    let text = format!("{} {}", short.dark_grey(), title);
    format!("{} {}{}", marker, text, tags.dark_grey())
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
                let color = std::io::stdout().is_terminal();
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
pub fn init<F>(debug: F, config: &Config, global: bool) -> Result<Report, Box<dyn Error>>
where
    F: Fn(&str),
{
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
