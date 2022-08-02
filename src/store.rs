use serde::{Deserialize, Serialize};
use crate::{item::Item, util::timestamp};

#[derive(Serialize, Deserialize)]
pub struct Store {
    items: Vec<Item>,
    last_write: u64,
}

impl Store {
    pub fn new(file: &str) -> Store {
        let serialized = match std::fs::read_to_string(file) {
            Ok(f) => f,
            Err(_err) => return Store{items: vec![], last_write: 0},
        };
        let _store: Store = serde_json::from_str(&serialized).unwrap();
        _store
    }

    pub fn write(&mut self, file: &str) {
        self.last_write = timestamp().as_secs();
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(file, serialized).unwrap();
    }

    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }
}
