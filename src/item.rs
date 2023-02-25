use core::fmt;

use crate::timestamp;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub children: Vec<String>,
    pub parents: Vec<String>,
    pub tags: Vec<String>,
    pub timetrack: Vec<u64>,
    pub content: String,
    pub timestamp: u64,   // creation timestamp
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
            "| id: {} | timestamp: {} | updated: {} \n| tags: {:?} \n| timetrack: {:?} \n| parents: {:?} \n| children: {:?} \n\n {}",
            self.id, self.timestamp, self.last_update, self.tags, self.timetrack, self.parents, self.children, self.content
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
        // prints only first line of the content
        return format!("|{}| {}", self.id, self.content.split("\n").next().unwrap());
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.timetrack.len() % 2 == 0 {
            self.timetrack.push(timestamp().as_secs());
            Ok(())
        } else {
            Err(format!(
                "{} runs already since {:?}",
                self.id.to_owned(),
                self.timetrack.last().unwrap()
            ))
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.timetrack.len() % 2 == 1 {
            self.timetrack.push(timestamp().as_secs());
            Ok(())
        } else {
            Err(format!("{} is not running.", self.id.to_owned()))
        }
    }

    pub fn set(&mut self, item: Item) {
        *self = item;
        self.last_update = timestamp().as_secs();
    }

    pub fn merge(&mut self, item: &mut Item) {
        self.content = item.content.to_string();
        self.tags.append(&mut item.tags);
        self.children.append(&mut item.children);
        self.parents.append(&mut item.parents);
    }
}
