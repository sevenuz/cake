use core::fmt;

use crate::timestamp;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub children: Vec<String>,
    pub parents: Vec<String>,
    pub tags: Vec<String>,
    pub timetrack: Vec<u64>,
    pub content: String,
    pub timestamp: u64, // creation timestamp
    pub last_update: u64, // last update timestamp
}

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
    ];

    nanoid!(3, &alphabet)
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}\ntimestamp: {}, updated: {}\ntags: {:?}\nparents: {:?}\nchildren: {:?}",
            self.print(), self.timestamp, self.last_update, self.tags, self.parents, self.children
        )
    }
}

impl Item {
    pub fn new(
        id: String,
        children: Vec<String>,
        parents: Vec<String>,
        tags: Vec<String>,
        content: String,
    ) -> Item {
        Item {
            id,
            children,
            parents,
            tags,
            content,
            timetrack: vec![],
            timestamp: timestamp().as_secs(),
            last_update: timestamp().as_secs(),
        }
    }

    // short form of fmt
    pub fn print(&self) -> String {
        return format!("|{:?}| {:?}", self.id, self.content);
    }
}
