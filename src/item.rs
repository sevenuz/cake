use chrono::{Local, TimeZone};
use core::fmt;
use termimad::crossterm::style::Stylize;

use crate::util;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    id: String,
    children: Vec<String>,
    parents: Vec<String>,
    tags: Vec<String>,
    timetrack: Vec<u64>,
    content: String,
    timestamp: u64,     // creation timestamp
    last_modified: u64, // last update timestamp
}

// format timestamp // TODO
fn ft(timestamp: u64) -> String {
    Local
        .timestamp_opt(i64::try_from(timestamp).unwrap(), 0)
        .unwrap()
        .to_string()
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "| id | {} \n| timestamp | {} \n| last modified | {} \n| tags | {:?} \n| timetrack | {:?} \n| parents | {:?} \n| children | {:?}\n\n{}",
            self.id, ft(self.timestamp), ft(self.last_modified), self.tags, self.timetrack, self.parents, self.children, self.content
        )
    }
}

impl Item {
    fn update_last_modified(&mut self) {
        self.last_modified = util::timestamp().as_secs();
    }

    pub fn new(id: String, children: Vec<String>, parents: Vec<String>, tags: Vec<String>) -> Self {
        Self {
            id,
            children,
            parents,
            tags,
            content: String::from(""),
            timetrack: vec![],
            timestamp: util::timestamp().as_secs(),
            last_modified: util::timestamp().as_secs(),
        }
    }

    pub fn id(&self) -> &String {
        return &self.id;
    }

    pub fn children(&self) -> &Vec<String> {
        return &self.children;
    }

    pub fn add_child(&mut self, child: &Self) {
        // TODO check if already child?
        self.children.push(child.id().to_owned());
        self.update_last_modified();
    }

    pub fn retain_child(&mut self, child: &Self) {
        self.children.retain(|s| !s.eq(child.id()));
        self.update_last_modified();
    }

    pub fn parents(&self) -> &Vec<String> {
        return &self.parents;
    }

    pub fn add_parent(&mut self, parent: &Self) {
        // TODO check if already parent?
        self.parents.push(parent.id().to_owned());
        self.update_last_modified();
    }

    pub fn retain_parent(&mut self, parent: &Self) {
        self.parents.retain(|s| !s.eq(parent.id()));
        self.update_last_modified();
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn append_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            if !self.tags.contains(&tag) {
                self.tags.push(tag.to_owned());
            }
        }
        self.update_last_modified();
    }

    pub fn content(&self) -> &String {
        return &self.content;
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.update_last_modified();
    }

    pub fn timestamp(&self) -> u64 {
        return self.timestamp;
    }

    // short form of fmt
    // the id is surrounded by spaces to reach spacer_len
    pub fn print(&self, spacer_len: usize, has_children: bool) -> String {
        // prints only first line of the content
        let border = if has_children { "\\" } else { "|" };
        return format!(
            "{}{}{}{} {}",
            border,
            util::space(&self.id, spacer_len),
            if self.is_started() {
                "*".dark_red()
            } else {
                "".white()
            },
            border,
            self.content.split("\n").next().unwrap()
        );
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
            self.update_last_modified();
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
            self.update_last_modified();
            Ok(())
        } else {
            Err(format!("{} is not running.", self.id.to_owned()))
        }
    }

    pub fn set(&mut self, item: Item) {
        *self = item;
        self.update_last_modified();
    }

    pub fn merge(&mut self, item: &mut Item) {
        self.content = item.content.to_string();
        self.tags.append(&mut item.tags);
        self.children.append(&mut item.children);
        self.parents.append(&mut item.parents);
        self.update_last_modified();
    }
}
