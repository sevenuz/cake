use crate::item::Item;
use crate::skin;
use crate::store::{inner::ItemView, RecState, Store, MAX_DEPTH};
use crate::util;
use crate::Selector;
use colored::*;
use std::error::Error;
use std::fs;

pub fn add<F>(
    debug: F,
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
    // TODO editor from settings
    let editor = "nvim";
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
            editor,
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
    println!("{}", store.get_item(&_id).unwrap());
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
            .append_tags(util::split_comma_cleanup(
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
    if selector.is_empty() {
        let items = store.get();
        let mut keys = items.keys().cloned().collect::<Vec<String>>();
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
        item_views = store.recursive_execute(&keys, &mut cycle, 0, max_depth, selector.rparents);
    } else {
        let ids = selector.get(store, false);
        item_views = store.recursive_execute(&ids, &mut cycle, 0, max_depth, selector.rparents);
    }

    // printing of results
    if long {
        for (i, item_view) in item_views.iter().enumerate() {
            if matches!(item_view.state, RecState::Reappearence) && item_view.depth > 0 {
                println!("{}", "### Recursion Warning ###".red());
            }
            debug(&format!("### raw ###\n{}", item_view.item.print_long(true)));

            // appends a dilimeter at the end if there are following items
            skin::build().print_text(
                &(item_view.item.to_string()
                    + if i + 1 < item_views.len() {
                        "\n---"
                    } else {
                        ""
                    }),
            );
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
                RecState::Normal => println!(
                    "{:indent$}{}",
                    "",
                    item_view.item.print(max_id_len, item_view.has_children),
                    indent = item_view.depth
                ),
                RecState::Reappearence if item_view.depth > 0 => {
                    print!(
                        "{:indent$}{}",
                        "",
                        item_view.item.print(max_id_len, item_view.has_children),
                        indent = item_view.depth
                    );
                    println!("{}", "### Recursion Warning ###".red());
                }
                _ => (),
            }
        }
    }

    Ok(())
}

pub fn show<F>(debug: F, path: &str) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("show: {:?}", path));
    let data = fs::read_to_string(path)?;
    skin::build().print_text(&data);
    Ok(())
}
