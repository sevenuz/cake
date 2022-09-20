use serde::{Deserialize, Serialize};
use nanoid::nanoid;
use crate::timestamp;

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub children: Vec<String>,
    pub parents: Vec<String>,
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
    pub fn new(id: String, children: Vec<String>, parents: Vec<String>, tags: Vec<String>, content: String) -> Item {
        Item{
            id,
            children,
            parents,
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
