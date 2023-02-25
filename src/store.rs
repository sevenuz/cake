pub use self::inner::Store;

pub mod inner {

    use std::collections::HashMap;

    use crate::{item::Item, util::timestamp};
    use serde::{Deserialize, Serialize};

    pub enum RecState {
        Normal,
        Cycle,
        Reappearence,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Store {
        items: HashMap<String, Item>,
        last_write: u64,
    }

    impl Store {
        // TODO better impl?
        fn check_existence(&self, ids: &Vec<String>) -> bool {
            for id in ids {
                if !self.items.contains_key(id) {
                    return false;
                }
            }
            true
        }

        fn set_relations(&mut self, item: &Item, add: bool) -> Result<(), &str> {
            if !self.check_existence(&item.parents) && !self.check_existence(&item.children) {
                return Err("Not all children or parents exist.");
            }
            for s in &item.parents {
                match self.get_item_mut(s) {
                    Some(i) => {
                        if add {
                            i.children.push(item.id.to_string());
                        } else {
                            i.children.retain(|s| !s.eq(&item.id))
                        }
                    }
                    None => (),
                }
            }
            for s in &item.children {
                match self.get_item_mut(s) {
                    Some(i) => {
                        if add {
                            i.parents.push(item.id.to_string());
                        } else {
                            i.parents.retain(|s| !s.eq(&item.id))
                        }
                    }
                    None => (),
                }
            }
            Ok(())
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

        pub fn edit(&mut self, mut item: Item, overwrite: bool) -> Result<(), &str> {
            if !self.items.contains_key(&item.id) {
                return Err("Key could not be found. Item was not updated.");
            }
            let id = item.id.clone();
            self.set_relations(&self.get_item(&item.id).unwrap().clone(), false); // delete old relations
            if overwrite {
                self.get_item_mut(&id).unwrap().set(item);
            } else {
                self.get_item_mut(&id).unwrap().merge(&mut item);
            }
            self.set_relations(&self.get_item(&id).unwrap().clone(), true)?; // set new relations
            Ok(())
        }

        pub fn add(&mut self, item: Item) -> Result<(), &str> {
            if self.items.contains_key(&item.id) {
                return Err(
                    "Key is already used. Set the \"edit\" flag if you want to update the item.",
                );
            }
            self.set_relations(&item, true);
            self.items.insert(item.id.to_owned(), item);
            Ok(())
        }

        pub fn remove(&mut self, id: &str, recursive: bool) {
            match self.items.remove(id) {
                Some(item) => {
                    self.set_relations(&item, false);
                    if recursive {
                        for rid in item.children {
                            self.remove(&rid, recursive);
                        }
                    }
                }
                None => (),
            }
        }

        pub fn get_item(&self, id: &str) -> Option<&Item> {
            return self.items.get(id);
        }

        pub fn get_item_mut(&mut self, id: &str) -> Option<&mut Item> {
            return self.items.get_mut(id);
        }

        pub fn get(&self) -> &HashMap<String, Item> {
            return &self.items;
        }

        // executes passed fn for every element in items, executing children recursively
        pub fn recursive_execute(
            &self,
            items: &Vec<String>,
            ids: &mut Vec<String>,
            f: fn(&Item, usize, RecState) -> (),
            depth: usize,
            max_depth: usize,
        ) {
            if depth == max_depth {
                return;
            }
            for s in items.iter() {
                let _item = self.items.get(s).unwrap();
                if !ids.contains(&s) {
                    ids.push(s.to_string());
                    f(_item, depth, RecState::Normal);
                    self.recursive_execute(&_item.children, ids, f, depth + 1, max_depth);
                } else {
                    f(
                        _item,
                        depth,
                        if ids.first().unwrap() == s {
                            RecState::Cycle
                        } else {
                            RecState::Reappearence
                        },
                    );
                }
            }
        }
    }
}
