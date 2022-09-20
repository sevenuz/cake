use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::{item::Item, util::timestamp};

#[derive(Serialize, Deserialize)]
pub struct Store {
    items: HashMap<String, Item>,
    last_write: u64,
}

impl Store {
    pub fn new(file: &str) -> Store {
        let serialized = match std::fs::read_to_string(file) {
            Ok(f) => f,
            Err(_err) => return Store{items: HashMap::new(), last_write: 0},
        };
        let _store: Store = serde_json::from_str(&serialized).unwrap();
        _store
    }

    pub fn write(&mut self, file: &str) {
        self.last_write = timestamp().as_secs();
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(file, serialized).unwrap();
    }

    pub fn add(&mut self, item: Item, edit: bool) -> Result<(), String>{
        if !edit && self.items.contains_key(&item.id) {
            return Err("Key is already used. Set the \"edit\" flag if you want to update the item.".to_string());
        }
        if edit && !self.items.contains_key(&item.id) {
            return Err("Key could not be found. Item was not updated.".to_string());
        }
        self.items.insert(item.id.to_owned(), item);
        Ok(())
    }

    pub fn remove(&mut self, id: &str, recursive: bool) {
        match self.items.remove(id) {
            Some(v) => {
                if recursive {
                    for rid in v.linked_items {
                        self.remove(&rid, recursive);
                    }
                }
            },
            None => ()
        }
    }

    pub fn get(&self) -> &HashMap<String, Item> {
        return &self.items;
    }

    pub fn recursive_execute(&self, items: &Vec<String>, ids: &mut Vec<String>, f: fn(&Item, usize) -> (), depth: usize, max_depth: usize) {
        if depth == max_depth {
            return;
        }
        for s in items.iter() {
            if !ids.contains(&s) && self.items.contains_key(s) {
                let _item = self.items.get(s).unwrap();
                ids.push(s.to_string());
                self.recursive_execute(&_item.linked_items, ids, f, depth + 1, max_depth);
                f(_item, depth);
            }
        }
    }

}
