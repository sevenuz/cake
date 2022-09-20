pub use self::inner::Store;

pub mod inner {

    use std::collections::HashMap;

    use crate::{item::Item, util::timestamp};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Store {
        items: HashMap<String, Item>,
        last_write: u64,
    }

    impl Store {
        fn edit_parent_of_children(&mut self, id: &String, children: &Vec<String>, add: bool) {
            for s in children {
                match self.items.get_mut(s) {
                    Some(item) => {
                        if add {
                            item.parents.push(id.to_string());
                        } else {
                            item.parents.retain(|s|!s.eq(id))
                        }
                    }
                    None => (),
                }
            }
        }

        fn edit_child_of_parents(&mut self, id: &String, parents: &Vec<String>, add: bool) {
            for s in parents {
                match self.items.get_mut(s) {
                    Some(item) => {
                        if add {
                            item.children.push(id.to_string());
                        } else {
                            item.children.retain(|s|!s.eq(id))
                        }
                    }
                    None => (),
                }
            }
        }

        pub fn new(file: &str) -> Store {
            let serialized = match std::fs::read_to_string(file) {
                Ok(f) => f,
                Err(_err) => {
                    return Store {
                        items: HashMap::new(),
                        last_write: 0,
                    }
                }
            };
            let _store: Store = serde_json::from_str(&serialized).unwrap();
            _store
        }

        pub fn write(&mut self, file: &str) {
            self.last_write = timestamp().as_secs();
            let serialized = serde_json::to_string_pretty(&self).unwrap();
            std::fs::write(file, serialized).unwrap();
        }

        pub fn add(&mut self, item: Item, edit: bool) -> Result<(), String> {
            if !edit && self.items.contains_key(&item.id) {
                return Err(
                    "Key is already used. Set the \"edit\" flag if you want to update the item."
                        .to_string(),
                );
            }
            if edit && !self.items.contains_key(&item.id) {
                return Err("Key could not be found. Item was not updated.".to_string());
            }
            self.edit_child_of_parents(&item.id, &item.parents, true);
            self.edit_parent_of_children(&item.id, &item.children, true);
            self.items.insert(item.id.to_owned(), item);
            Ok(())
        }

        pub fn remove(&mut self, id: &str, recursive: bool) {
            match self.items.remove(id) {
                Some(v) => {
                    self.edit_child_of_parents(&v.id, &v.parents, false);
                    self.edit_parent_of_children(&v.id, &v.children, false);
                    if recursive {
                        for rid in v.children {
                            self.remove(&rid, recursive);
                        }
                    }
                }
                None => (),
            }
        }

        pub fn get(&self) -> &HashMap<String, Item> {
            return &self.items;
        }

        pub fn recursive_execute(
            &self,
            items: &Vec<String>,
            ids: &mut Vec<String>,
            f: fn(&Item, usize) -> (),
            depth: usize,
            max_depth: usize,
        ) {
            if depth == max_depth {
                return;
            }
            for s in items.iter() {
                if !ids.contains(&s) && self.items.contains_key(s) {
                    let _item = self.items.get(s).unwrap();
                    ids.push(s.to_string());
                    f(_item, depth);
                    self.recursive_execute(&_item.children, ids, f, depth + 1, max_depth);
                }
            }
        }
    }
}
