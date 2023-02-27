use crate::item::Item;
use crate::store::Store;
use crate::util::*;
use std::error::Error;

pub fn add<F>(
    debug: F,
    store: &mut Store,
    ids: Vec<String>,
    children: Vec<String>,
    parents: Vec<String>,
    tags: Vec<String>,
    content: String,
    edit: bool,
    overwrite: bool,
) -> Result<(), Box<dyn Error>> where F: FnOnce(&str) {
    // TODO editor from settings
    let editor = "vim";
    let _id = if ids.is_empty() {
        generate_id()
    } else {
        ids.first().unwrap().to_string()
    };
    let mut item = Item::new(_id.clone(), children, parents, tags);
    store.check_existence(&item, edit)?;
    item.content = if content.is_empty() {
        input_from_external_editor(
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
