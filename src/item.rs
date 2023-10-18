use chrono::{DateTime, Local, TimeZone};
use core::fmt;
use std::str::FromStr;
use termimad::crossterm::style::Stylize;

use crate::util;
use serde::{Deserialize, Serialize};

const PREFIX_ID: &str = "| id | ";
const PREFIX_TIMESTAMP: &str = "| timestamp | ";
const PREFIX_LAST_MODIFIED: &str = "| last modified | ";
const PREFIX_TAGS: &str = "| tags | ";
const PREFIX_TIMETRACK: &str = "| timetrack | ";
const PREFIX_PARENTS: &str = "| parents | ";
const PREFIX_CHILDREN: &str = "| children | ";

const DATE_FORMAT: &str = "%d.%m.%Y  %H:%M";

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    id: String,
    children: Vec<String>,
    parents: Vec<String>,
    tags: Vec<String>,
    timetrack: Vec<i64>,
    content: String,
    timestamp: i64,     // creation timestamp
    last_modified: i64, // last update timestamp
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseItemError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_to_str() {
        assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
        assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
    }

    #[test]
    fn test_ft() {
        assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
        assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
    }

    #[test]
    fn test_tf() {
        assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
        assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
    }
}

fn str_to_vec(s: &str) -> Vec<String> {
    s.split(", ").map(|v| v.to_string()).collect()
}

fn vec_to_str<T>(v: &Vec<T>) -> String
where
    T: std::fmt::Display,
{
    let mut res: String = "".to_string();
    for s in v {
        res += &s.to_string();
        res += ", ";
    }
    if v.is_empty() {
        "".to_string()
    } else {
        // remove last comma
        res.strip_suffix(", ").unwrap().to_string()
    }
}

// show timestamp in hours, minutes, seconds
fn hm(timestamp: i64) -> String {
    let hours = timestamp / 60 / 60;
    let minutes = (timestamp - hours * 60) / 60;
    let mut res = "".to_string();
    if hours > 0 {
        res += &format!("{}h", hours);
    }
    if minutes > 0 {
        res += &format!("{}m", minutes);
    }
    if hours + minutes == 0 {
        res += &format!("{}s", timestamp);
    }
    res
}

// format timestamp // TODO
fn ft(timestamp: i64) -> String {
    Local
        .timestamp_opt(i64::try_from(timestamp).unwrap(), 0)
        .unwrap()
        .format(DATE_FORMAT)
        .to_string()
}

// from the formatted string of fn ft, this parses the unix timestamp
fn tf(s: &str) -> Result<i64, ParseItemError> {
    let stripped_timestamp = s;
    let dt_timestamp = DateTime::parse_from_str(stripped_timestamp, DATE_FORMAT)
        .ok()
        .ok_or(ParseItemError)?;
    return Ok(dt_timestamp.timestamp());
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.print_long(false))
    }
}

fn pollish(s: &str, prefix: &str) -> Result<String, ParseItemError> {
    Ok(s.strip_prefix(prefix)
        .ok_or(ParseItemError)?
        .strip_suffix("|")
        .ok_or(ParseItemError)?
        .to_string())
}

// https://doc.rust-lang.org/std/str/trait.FromStr.html
impl FromStr for Item {
    type Err = ParseItemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let id: String;
        if let Some(raw_id) = lines.next() {
            id = pollish(raw_id, PREFIX_ID)?;
        } else {
            return Err(ParseItemError);
        };
        let timestamp: i64;
        if let Some(raw_timestamp) = lines.next() {
            timestamp = tf(&pollish(raw_timestamp, PREFIX_TIMESTAMP)?)?;
        } else {
            return Err(ParseItemError);
        };
        let last_modified: i64;
        if let Some(raw_last_modified) = lines.next() {
            last_modified = tf(&pollish(raw_last_modified, PREFIX_LAST_MODIFIED)?)?;
        } else {
            return Err(ParseItemError);
        };
        let tags: Vec<String>;
        if let Some(raw_tags) = lines.next() {
            tags = str_to_vec(&pollish(raw_tags, PREFIX_TAGS)?);
        } else {
            return Err(ParseItemError);
        };
        let timetrack: Vec<i64>;
        if let Some(raw_timetrack) = lines.next() {
            timetrack = pollish(raw_timetrack, PREFIX_TIMETRACK)?
                .split(", ")
                .map(|a| (tf(a).unwrap())) // TODO unwrap() o.0
                .collect();
        } else {
            return Err(ParseItemError);
        };
        let parents: Vec<String>;
        if let Some(raw_parents) = lines.next() {
            parents = str_to_vec(&pollish(raw_parents, PREFIX_PARENTS)?);
        } else {
            return Err(ParseItemError);
        };
        let children: Vec<String>;
        if let Some(raw_children) = lines.next() {
            children = str_to_vec(&pollish(raw_children, PREFIX_CHILDREN)?);
        } else {
            return Err(ParseItemError);
        };
        // the two blank lines
        lines.next();
        lines.next();
        let content = lines.collect();
        Ok(Item {
            id,
            timestamp,
            last_modified,
            tags,
            timetrack,
            parents,
            children,
            content,
        })
    }
}

impl Item {
    fn update_last_modified(&mut self) {
        self.last_modified = util::timestamp();
    }

    pub fn new(id: String, children: Vec<String>, parents: Vec<String>, tags: Vec<String>) -> Self {
        Self {
            id,
            children,
            parents,
            tags,
            content: String::from(""),
            timetrack: vec![],
            timestamp: util::timestamp(),
            last_modified: util::timestamp(),
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

    pub fn timestamp(&self) -> i64 {
        return self.timestamp;
    }

    // short info about the item
    // prints only first line of the content and id
    // the id is surrounded by spaces to reach spacer_len
    pub fn print(&self, spacer_len: usize, has_children: bool) -> String {
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

    pub fn print_long(&self, serialize: bool) -> String {
        let tt;
        if serialize {
            tt = self
                .timetrack
                .iter()
                .map(|t| ft(*t))
                .collect::<Vec<String>>();
        } else {
            let even = self
                .timetrack
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 0);
            let odd = self
                .timetrack
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 1);
            tt = even
                .zip(odd)
                .map(|((_, first), (_, second))| second - first)
                .map(|t| hm(t))
                .collect::<Vec<String>>();
        }
        let res = format!(
            "{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n\n{}",
            PREFIX_ID,
            self.id,
            PREFIX_TIMESTAMP,
            ft(self.timestamp),
            PREFIX_LAST_MODIFIED,
            ft(self.last_modified),
            PREFIX_TAGS,
            vec_to_str(&self.tags),
            PREFIX_TIMETRACK,
            vec_to_str(&tt),
            PREFIX_PARENTS,
            vec_to_str(&self.parents),
            PREFIX_CHILDREN,
            vec_to_str(&self.children),
            self.content
        );
        res
    }

    pub fn is_started(&self) -> bool {
        return self.timetrack.len() % 2 == 1;
    }

    pub fn is_stopped(&self) -> bool {
        return self.timetrack.len() % 2 == 0;
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_stopped() {
            self.timetrack.push(util::timestamp());
            self.update_last_modified();
            Ok(())
        } else {
            Err(format!(
                "{} runs already since {}",
                self.id.to_owned(),
                ft(*self.timetrack.last().unwrap())
            ))
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.is_started() {
            self.timetrack.push(util::timestamp());
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
