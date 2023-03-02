use crate::item::Item;
use crate::store::{RecState, Store};
use crate::util;
use colored::*;
use std::error::Error;

// default filter
#[derive(Debug)]
pub struct Selector {
    ids: Vec<String>,
    children: Vec<String>,
    parents: Vec<String>,
    tags: Vec<String>,
    before: Option<u64>, // time in seconds relative to now
    after: Option<u64>,  // time in seconds relative to now
    started: bool,
    stopped: bool,
    rparents: bool,  // recursive for parents
    rchildren: bool, // recursive for children
}

impl Selector {
    pub fn new(
        ids: &Option<String>,
        children: &Option<String>,
        parents: &Option<String>,
        tags: &Option<String>,
        before: &Option<String>,
        after: &Option<String>,
        started: &bool,
        stopped: &bool,
        recursive: &u8,
    ) -> Result<Selector, Box<dyn Error>> {
        Ok(Selector {
            ids: util::split_comma_cleanup(ids.to_owned().unwrap_or("".to_string())),
            children: util::split_comma_cleanup(children.to_owned().unwrap_or("".to_string())),
            parents: util::split_comma_cleanup(parents.to_owned().unwrap_or("".to_string())),
            tags: util::split_comma_cleanup(tags.to_owned().unwrap_or("".to_string())),
            before: util::parse_time(&before.to_owned().unwrap_or("".to_string()))?,
            after: util::parse_time(&after.to_owned().unwrap_or("".to_string()))?,
            started: *started,
            stopped: *stopped,
            rparents: *recursive > 1, // -rr only parents, -rrr both
            rchildren: *recursive == 1 || *recursive > 2, // -r only children, -rrr both
        })
    }
}

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
    debug(&format!(
        "add {:?}, content: {:?}, edit: {:?}, overwrite: {:?}",
        selector, content, edit, overwrite
    ).clone());
    // TODO editor from settings
    let editor = "vim";
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
    item.content = if content.is_empty() {
        util::input_from_external_editor(
            editor,
            if edit {
                Some(&store.get_item(&_id.to_string()).unwrap().content)
            } else {
                None
            },
        )
        .unwrap()
    } else {
        content
    };
    debug(&item.content);

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
    match selector.ids.first() {
        Some(id) => {
            store.remove(id, selector.rchildren)?;
            println!("{:?} removed.", id); // TODO print count of deleted items
            Ok(())
        }
        None => Err("You have to provide an id of a todo".into()), // TODO
    }
}

pub fn start<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("start {:?}", selector));
    match selector.ids.first() {
        Some(id) => {
            store
                .get_item_mut(id)
                .expect("Could not found id")
                .start()?;
            println!("{:?} started.", id);
            Ok(())
        }
        None => Err("You have to provide an id of a todo".into()), // TODO
    }
}

pub fn stop<F>(debug: F, store: &mut Store, selector: Selector) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str),
{
    debug(&format!("stop {:?}", selector));
    match selector.ids.first() {
        Some(id) => {
            store.get_item_mut(id).expect("Could not found id").stop()?;
            println!("{:?} started.", id);
            Ok(())
        }
        None => Err("You have to provide an id of a todo".into()), // TODO
    }
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
    let max_depth = if selector.rchildren {
        10 /*std::usize::MAX*/
    } else {
        1
    };

    fn print_line(line: String, depth: usize, state: &RecState) {
        match state {
            RecState::Normal => println!("{:indent$}{}", "", line, indent = depth),
            RecState::Reappearence if depth > 0 => {
                println!("{:indent$}{}", "", line.bright_green(), indent = depth)
            }
            _ => (),
        }
    }

    fn prl(item: &Item, depth: usize, state: RecState) {
        // double indention for visuality reasons
        let indent = 10 * depth;
        for line in item.to_string().split("\n") {
            print_line(line.to_string(), indent, &state);
        }
        if !(matches!(state, RecState::Reappearence) && depth == 0) {
            println!(
                "{}",
                "=====================================================".red()
            );
        }
    }

    fn prs(item: &Item, depth: usize, state: RecState) {
        print_line(item.print(), depth, &state);
    }

    match selector.ids.first() {
        Some(id) => {
            store.check_id(id, true)?;
            let keys = vec![id.to_owned()];
            store.recursive_execute(
                &keys,
                &mut cycle,
                if long { prl } else { prs },
                0,
                max_depth,
            );
            Ok(())
        }
        None => {
            let items = store.get();
            let mut keys = items.keys().cloned().collect::<Vec<String>>();
            // sort output from old to new
            keys.sort_by(|a, b| {
                items
                    .get(a)
                    .unwrap()
                    .timestamp
                    .cmp(&items.get(b).unwrap().timestamp)
            });
            // sort output by amount of parents. Zero parents first
            keys.sort_by(|a, b| {
                items
                    .get(a)
                    .unwrap()
                    .parents
                    .len()
                    .cmp(&items.get(b).unwrap().parents.len())
            });
            store.recursive_execute(
                &keys,
                &mut cycle,
                if long { prl } else { prs },
                0,
                max_depth,
            );
            Ok(())
        }
    }
}
