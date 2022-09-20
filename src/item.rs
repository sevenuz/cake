use serde::{Deserialize, Serialize};
use nanoid::nanoid;
use crate::timestamp;

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub linked_items: Vec<String>,
    pub tags: Vec<String>,
    pub content: String,
    pub timestamp: u64
}

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
    ];

    nanoid!(3, &alphabet)
}

impl Item {
    pub fn new(id: String, linked_items: Vec<String>, tags: Vec<String>, content: String) -> Item {
        Item{
            id,
            linked_items,
            tags,
            content,
            timestamp: timestamp().as_secs()
        }
    }

    pub fn print(&self) -> String {
        return format!("|{:?}| {:?}", self.id, self.content);
    }

    pub fn print_long(&self) -> String {
        return format!("|{:?}| {:?}", self.id, self.content); // TODO long form
    }
}
