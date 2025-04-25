use crate::config::Config;
use crate::item::Item;
use crate::store::{inner::ItemView, RecState, Store, MAX_DEPTH};
use crate::view;
use crate::Selector;
use crate::{git, util};
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;
use termimad::crossterm::style::Stylize;

pub fn add<F>(
    debug: F,
    config: &Config,
    store: &mut Store,
    selector: Selector,
    content: String,
    edit: bool,
    overwrite: bool,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(
        &format!(
            "add {:?}, content: {:?}, edit: {:?}, overwrite: {:?}",
            selector, content, edit, overwrite
        )
        .clone(),
    );
    let _id = if selector.ids.is_empty() {
        util::generate_id()
    } else {
        selector.ids.first().unwrap().to_string()
    };
    let mut item = Item::new(
        _id.clone(),
        selector.children,
        selector.parents,
        selector.tags,
    );
    store.check_existence(&item, edit)?;
    item.set_content(if content.is_empty() {
        util::input_from_external_editor(
            &config.editor,
            if edit {
                Some(&store.get_item(&_id.to_string()).unwrap().content())
            } else {
                None
            },
        )
        .unwrap()
    } else {
        content
    });
    debug(&item.content());

    if edit {
        store.edit(item, overwrite)?;
    } else {
        store.add(item)?;
    }
    view::print(&config, store.get_item(&_id).unwrap().print_long(false))?;
    Ok(())
}

pub fn remove<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("remove {:?}", selector));
    let ids = selector.get(store, true);
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
    let ids = selector.get(store, true);
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
    let ids = selector.get(store, true);
    for id in &ids {
        store
            .get_item_mut(id)
            .expect("Could not found id")
            .start()?;
    }
    println!("{} started.", ids.len());
    Ok(())
}

pub fn stop<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("stop {:?}", selector));
    let ids = selector.get(store, true);
    for id in &ids {
        store.get_item_mut(id).expect("Could not found id").stop()?;
    }
    println!("{} stopped.", ids.len());
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

pub fn init<F>(debug: F, config: &Config, git: bool, remote: bool) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("init: git {:?}, remote {:?}", git, remote));
    if git {
        if config.disable_git {
            return Err(Box::from(
                "You have git support disabled. Check 'cake config'.",
            ));
        }
        if !git::is_repo() {
            return Err(Box::from(
                "You can use 'cake init --git' only in an existing repository.",
            ));
        }
        if git::check_if_branch_exists(config)? {
            return Err(Box::from(
                "The branch '".to_owned() + &config.git_branch_name + "' already exists. Abort...",
            ));
        }
        let current_branch = git::current_branch_name()?;
        debug(&format!("init: current branch {:?}", current_branch));
        debug(&format!("init: stash"));
        git::stash()?;
        debug(&format!("init: create {}", config.git_branch_name));
        git::create_branch(config)?;
        debug(&format!("init: checkout {}", config.git_branch_name));
        git::checkout_branch(&config.git_branch_name)?;
        let p = Path::new(&config.save_file_name);
        debug(&format!("init: new file {:?}", p));
        if let Err(e) = File::create_new(p) {
            println!(
                "I guess you already have a cake file in your repo o.0 \n {} \n{}",
                e.to_string().yellow().italic(),
                "So we are just using this one now. You can remove it from your other branches."
            );
        } else {
            println!(
                "New {} file created at branch {} :)",
                config.save_file_name, config.git_branch_name
            );
            debug(&format!("init: add {}", config.save_file_name));
            git::add(config)?;
            debug(&format!("init: commit {}", config.save_file_name));
            git::commit("init cake file")?;
        }
        if remote {
            debug(&format!(
                "init: push --set-upstream {} {}",
                config.git_remote_name, config.git_branch_name
            ));
            git::push_set_upstream(config)?;
        }
        debug(&format!("init: checkout {}", current_branch));
        git::checkout_branch(&current_branch)?;
        debug(&format!("init: stash pop"));
        git::stash_pop()?;
    } else {
        let p = Path::new(&config.save_file_name);
        debug(&format!("init: new file {:?}", p));
        File::create_new(p)?;
        println!("New {} file created :)", config.save_file_name);
    }
    Ok(())
}
