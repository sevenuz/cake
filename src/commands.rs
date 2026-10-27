use crate::config::Config;
use crate::item::{Item, Status};
use crate::store::{inner::ItemView, RecState, Store, MAX_DEPTH};
use crate::view;
use crate::Selector;
use crate::{git, util};
use std::error::Error;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;
use termimad::crossterm::style::Stylize;

/// fields of a new item besides selector and content
pub struct AddOptions {
    pub note: bool,
    pub due: Option<String>,
    pub priority: Option<u64>,
    pub blocked_by: Vec<String>,
    pub related: Vec<String>,
}

/// adds an item; without content the editor opens, which needs a terminal
pub fn add<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
    content: Option<String>,
    options: AddOptions,
) -> Result<(), Box<dyn Error>>
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
    if let Some(missing) = options
        .blocked_by
        .iter()
        .chain(options.related.iter())
        .find(|i| store.get_item(i).is_none())
    {
        return Err(format!("{} not found.", missing).into());
    }
    if options.note {
        item.set_note();
    }
    item.set_due(options.due)?;
    item.set_priority(options.priority)?;
    item.set_blocked_by(options.blocked_by);
    item.set_related(options.related);
    item.set_content(match content {
        Some(c) => c,
        None if std::io::stdin().is_terminal() => {
            util::input_from_external_editor(&config.editor, None)?
        }
        None => return Err("No content, use -m TEXT or --body-file FILE (- for stdin).".into()),
    });
    debug(&item.content());
    store.add(item)?;
    view::print(&config, store.get_item(&id).unwrap().print_long(false))?;
    Ok(())
}

/// opens the whole item file (frontmatter and body) in the editor
pub fn edit<F>(debug: F, config: &Config, store: &mut Store, id: &str) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("edit {:?}", id));
    let old = store.get_item(id).ok_or(format!("{} not found.", id))?;
    if !std::io::stdin().is_terminal() {
        return Err("cake edit needs a terminal.".into());
    }
    let text = old.to_markdown();
    let edited = util::input_from_external_editor(&config.editor, Some(&text))?;
    if edited == text {
        println!("{} unchanged.", id);
        return Ok(());
    }
    let item = Item::from_markdown(&edited, id, old.timestamp())?;
    if item.id() != id {
        return Err("Changing the id is not supported.".into());
    }
    store.check_existence(&item, true)?;
    store.edit(item)?;
    view::print(&config, store.get_item(id).unwrap().print_long(false))?;
    Ok(())
}

/// ids for commands that change items: refuses an empty selector (it would
/// match everything) and unknown ids
fn select(store: &Store, selector: &Selector) -> Result<Vec<String>, Box<dyn Error>> {
    if selector.is_empty() {
        return Err("No items selected, give ids or selectors.".into());
    }
    if let Some(id) = selector.ids.iter().find(|id| store.get_item(id).is_none()) {
        return Err(format!("{} not found.", id).into());
    }
    Ok(selector.get(store, true))
}

pub fn remove<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("remove {:?}", selector));
    let ids = select(store, &selector)?;
    for id in &ids {
        store.remove(id)?;
    }
    println!("{} removed.", ids.len());
    Ok(())
}

pub fn tag<F>(
    debug: F,
    store: &mut Store,
    selector: Selector,
    tags: &Option<String>,
) -> Result<(), Box<dyn Error>>
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
    println!("{} tagged.", ids.len());
    Ok(())
}

pub fn start<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
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
    println!("{} started.", ids.len());
    Ok(())
}

/// stops the selected items, without selector the running ones
pub fn stop<F>(debug: F, store: &mut Store, mut selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("stop {:?}", selector));
    if selector.is_empty() {
        selector.started = true;
    }
    let ids = select(store, &selector)?;
    for id in &ids {
        store.get_item_mut(id).expect("Could not found id").stop()?;
    }
    println!("{} stopped.", ids.len());
    Ok(())
}

/// re-parents an item, "-" detaches it
pub fn mv<F>(debug: F, store: &mut Store, id: &str, parent: &str) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("mv {:?} {:?}", id, parent));
    let parent = if parent == "-" { None } else { Some(parent) };
    store.move_item(id, parent)?;
    match parent {
        Some(p) => println!("{} moved under {}.", id, p),
        None => println!("{} detached.", id),
    }
    Ok(())
}

pub fn set_status<F>(
    debug: F,
    store: &mut Store,
    selector: Selector,
    status: Status,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("set_status {:?} {:?}", selector, status));
    let ids = select(store, &selector)?;
    for id in &ids {
        store.get_item_mut(id).unwrap().set_status(status)?;
    }
    println!("{} {}.", ids.len(), status.as_str());
    Ok(())
}

pub fn list<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
    long: bool,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("list {:?} long: {:?}", selector, long));

    let mut cycle: Vec<String> = vec![];
    let item_views: Vec<ItemView>;
    let max_depth = if selector.rchildren { MAX_DEPTH } else { 1 };

    // TODO recursive for both: rparents, rchildren
    // TODO shows same item as a child on -rrr
    let items = store.get();
    let mut keys = selector.get(store, false);
    // sort output from old to new
    keys.sort_by(|a, b| {
        items
            .get(a)
            .unwrap()
            .timestamp()
            .cmp(&items.get(b).unwrap().timestamp())
    });
    // sort output by amount of parents. Zero parents first
    keys.sort_by(|a, b| {
        items
            .get(a)
            .unwrap()
            .parents()
            .len()
            .cmp(&items.get(b).unwrap().parents().len())
    });
    // get the recursive ItemView to print with indention
    item_views = store
        .recursive_execute(&keys, &mut cycle, 0, max_depth, selector.rparents)
        .iter()
        .filter(|iv| {
            // filter exclude tags again, because recursive execution is not filtering in selector
            selector.exclude(iv.item.id(), store)
        })
        .map(|iv| iv.to_owned())
        .collect();

    // printing of results
    let mut text: String = "".to_string();
    if long {
        for (i, item_view) in item_views.iter().enumerate() {
            if matches!(item_view.state, RecState::Reappearence) && item_view.depth > 0 {
                text = text + &format!("{}", "### Recursion Warning ###".red());
            }
            debug(&format!("### raw ###\n{}", item_view.item.print_long(true)));

            // appends a dilimeter at the end if there are following items
            text = text
                + &(item_view.item.to_string()
                    + if i + 1 < item_views.len() {
                        "\n---\n"
                    } else {
                        ""
                    });
        }
    } else {
        // find maximun id len
        let max_id_len = item_views.iter().fold(0, |a, b| {
            if a > b.item.id().len() {
                a
            } else {
                b.item.id().len()
            }
        });
        for item_view in item_views {
            match item_view.state {
                RecState::Normal => {
                    text = text
                        + &format!(
                            "{:indent$} {}\n",
                            "‎",
                            item_view.item.print(max_id_len, item_view.has_children),
                            indent = if item_view.has_children {
                                item_view.depth + 1
                            } else {
                                item_view.depth
                            }
                        );
                    debug(&format!("indention {:?}", item_view.depth));
                }
                RecState::Reappearence if item_view.depth > 0 => {
                    text = text
                        + &format!(
                            "{:indent$} {}",
                            "‎",
                            item_view.item.print(max_id_len, item_view.has_children),
                            indent = if item_view.has_children {
                                item_view.depth + 1
                            } else {
                                item_view.depth
                            }
                        );
                    text = text + &format!(" {}\n", "### Reappearence Warning ###".red());
                    debug(&format!("indention {:?}", item_view.depth));
                }
                _ => (),
            }
        }
    }
    view::print(&config, text)?;

    Ok(())
}

pub fn show<F>(debug: F, config: &Config, path: &str) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("show: {:?}", path));
    let data = fs::read_to_string(path)?;
    view::print(&config, data)?;
    Ok(())
}

/// Creates the store: inside a git repo a worktree of `config.git_branch` at
/// `<repo>/.cake` (attached if the branch exists locally or on the remote),
/// otherwise a plain `.cake` dir in the current directory.
/// With `global`, the global store becomes a git repo on `main`.
pub fn init<F>(debug: F, config: &Config, global: bool) -> Result<(), Box<dyn Error>>
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
            return Err(format!("There is already a store at {}.", p.display()).into());
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
        println!("New global store {} created :)", p.display());
        return Ok(());
    }
    let cwd = std::env::current_dir()?;
    if config.disable_git || !git::is_repo(&cwd) {
        let p = cwd.join(util::STORE_DIR);
        debug(&format!("init: new store {:?}", p));
        if p.join("items").is_dir() {
            return Err("There is already a store here.".into());
        }
        create_store_files(&p)?;
        println!("New store {} created :)", p.display());
        return Ok(());
    }
    let repo = git::toplevel(&cwd)?;
    let p = repo.join(util::STORE_DIR);
    if p.exists() {
        return Err(format!("{} already exists.", p.display()).into());
    }
    debug(&format!("init: worktree {:?} on {}", p, config.git_branch));
    let attached = git::add_worktree(&repo, &p, &config.git_branch, &config.git_remote_name)?;
    git::exclude(&repo, &format!("/{}/", util::STORE_DIR))?;
    if attached {
        fs::create_dir_all(p.join("items"))?;
        println!("Attached store {} on branch {} :)", p.display(), config.git_branch);
    } else {
        let files = create_store_files(&p)?;
        git::commit(&p, &files, "cake: init")?;
        println!("New store {} created on branch {} :)", p.display(), config.git_branch);
    }
    Ok(())
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
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    if config.disable_git || !git::is_store_repo(root) {
        return Err(format!("{} is not a git store, see `cake help init`.", root.display()).into());
    }
    if abort {
        debug("sync: abort");
        git::sync_abort(root)?;
        println!("Merge aborted.");
    } else if cont {
        debug("sync: continue");
        git::sync_continue(root)?;
        println!("Synced.");
    } else if let Some(remote) = publish {
        let remote = if remote.is_empty() { &config.git_remote_name } else { remote };
        debug(&format!("sync: publish to {}", remote));
        git::publish(root, remote)?;
        println!("Published to {}.", remote);
    } else {
        debug("sync");
        git::sync(root)?;
        println!("Synced.");
    }
    Ok(())
}

pub fn config<F>(debug: F, config: &Config) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!(
        "config: open editor {:?} with file {:?}",
        config.editor,
        util::config_file()?
    ));
    Command::new(&config.editor)
        .arg(util::config_file()?)
        .status()?;
    Ok(())
}
