pub use self::inner::Store;

pub mod inner {

    use std::collections::HashMap;
    use std::error::Error;

    use crate::{item::Item, util::timestamp};
    use serde::{Deserialize, Serialize};

    pub enum RecState {
        Normal,
        Reappearence,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Store {
        items: HashMap<String, Item>,
        last_write: u64,
    }

    impl Store {
        pub fn check_id(&self, id: &String, itself: bool) -> Result<(), &'static str> {
            if itself && !self.items.contains_key(id) {
                return Err("Item does not exist.");
            } else if !itself && self.items.contains_key(id) {
                return Err("Item already exists. Set the \"edit\" flag if you want to update the item.");
            }
            Ok(())
        }

        pub fn check_existence(&self, item: &Item, itself: bool) -> Result<(), &'static str> {
            self.check_id(&item.id, itself)?;
            for id in &item.children {
                if !self.items.contains_key(id) {
                    return Err("Not all children exist.");
                }
            }
            for id in &item.parents {
                if !self.items.contains_key(id) {
                    return Err("Not all parents exist.");
                }
            }
            Ok(())
        }

        fn set_relations(&mut self, item: &Item, add: bool) -> Result<(), Box<dyn Error>> {
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

        pub fn new(file: &str) -> Result<Store, Box<dyn Error>> {
            let serialized = match std::fs::read_to_string(file) {
                Ok(f) => f,
                Err(_err) => {
                    return Ok(Store {
                        items: HashMap::new(),
                        last_write: 0,
                    })
                }
            };
            let _store: Store = serde_json::from_str(&serialized)?;
            Ok(_store)
        }

        pub fn write(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
            self.last_write = timestamp().as_secs();
            let serialized = serde_json::to_string_pretty(&self)?;
            std::fs::write(file, serialized)?;
            Ok(())
        }

        // existence is checked in command
        pub fn edit(&mut self, mut item: Item, overwrite: bool) -> Result<(), Box<dyn Error>> {
            // delete old relations
            self.set_relations(&self.get_item(&item.id).unwrap().clone(), false)?;
            if overwrite {
                self.get_item_mut(&item.id).unwrap().set(item.clone());
            } else {
                self.get_item_mut(&item.id).unwrap().merge(&mut item);
            }
            // set new relations
            self.set_relations(&self.get_item(&item.id).unwrap().clone(), true)?;
            Ok(())
        }

        // existence is checked in command
        pub fn add(&mut self, item: Item) -> Result<(), Box<dyn Error>> {
            self.set_relations(&item, true)?;
            self.items.insert(item.id.to_owned(), item);
            Ok(())
        }

        pub fn remove(&mut self, id: &str, recursive: bool) -> Result<(), Box<dyn Error>> {
            match self.items.remove(id) {
                Some(item) => {
                    self.set_relations(&item, false)?;
                    if recursive {
                        for rid in item.children {
                            self.remove(&rid, recursive)?;
                        }
                    }
                }
                None => return Err("Could not found id".into()),
            }
            Ok(())
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
        // existence in not checked, so it can panic
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
                    f(_item, depth, RecState::Reappearence);
                }
            }
        }
    }
}
