use chrono::{Local, TimeZone};
use core::fmt;

use crate::util;
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

// format timestamp // TODO
fn ft(timestamp: u64) -> String {
    Local.timestamp_opt(i64::try_from(timestamp).unwrap(), 0)
        .unwrap()
        .to_string()
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "| id: {} | timestamp: {} | updated: {} \n| tags: {:?} \n| timetrack: {:?} \n| parents: {:?} \n| children: {:?} \n\n {}",
            self.id, ft(self.timestamp), ft(self.last_update), self.tags, self.timetrack, self.parents, self.children, self.content
        )
    }
}

impl Item {
    pub fn new(id: String, children: Vec<String>, parents: Vec<String>, tags: Vec<String>) -> Item {
        Item {
            id,
            children,
            parents,
            tags,
            content: String::from(""),
            timetrack: vec![],
            timestamp: util::timestamp().as_secs(),
            last_update: util::timestamp().as_secs(),
        }
    }

    // short form of fmt
    pub fn print(&self) -> String {
        // prints only first line of the content
        return format!("|{}| {}", self.id, self.content.split("\n").next().unwrap());
    }

    pub fn is_started(&self) -> bool {
        return self.timetrack.len() % 2 == 1;
    }

    pub fn is_stopped(&self) -> bool {
        return self.timetrack.len() % 2 == 0;
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_stopped() {
            self.timetrack.push(util::timestamp().as_secs());
            Ok(())
        } else {
            Err(format!(
                "{} runs already since {:?}",
                self.id.to_owned(),
                ft(*self.timetrack.last().unwrap())
            ))
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.is_started() {
            self.timetrack.push(util::timestamp().as_secs());
            Ok(())
        } else {
            Err(format!("{} is not running.", self.id.to_owned()))
        }
    }

    pub fn set(&mut self, item: Item) {
        *self = item;
        self.last_update = util::timestamp().as_secs();
    }

    pub fn merge(&mut self, item: &mut Item) {
        self.content = item.content.to_string();
        self.tags.append(&mut item.tags);
        self.children.append(&mut item.children);
        self.parents.append(&mut item.parents);
        self.last_update = util::timestamp().as_secs();
    }
}
