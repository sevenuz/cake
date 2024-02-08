use crate::util;
use chrono::{DateTime, Local, TimeZone};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use termimad::crossterm::style::Stylize;

#[cfg(test)]
mod tests;

const PREFIX_ID: &str = "| id | ";
const TABLE_HEADER_DELIMITER: &str = "|---|---|";
const PREFIX_TIMESTAMP: &str = "| timestamp | ";
const PREFIX_LAST_MODIFIED: &str = "| last modified | ";
const PREFIX_TAGS: &str = "| tags | ";
const PREFIX_TIMETRACK: &str = "| timetrack | ";
const PREFIX_PARENTS: &str = "| parents | ";
const PREFIX_CHILDREN: &str = "| children | ";

// DO NOT CHANGE IT, it will break tf
const DATE_FORMAT: &str = "%c %z";

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

// https://users.rust-lang.org/t/what-is-stderror-and-how-exactly-does-propagate-errors/86267
// impl for StdError which is alias of std::error::Error
// needed for the Box<dyn Err>
const PARSE_ITEM_ERROR_MSG: &str = "Invalid string to create an item";
impl std::error::Error for ParseItemError {
    fn description(&self) -> &str {
        PARSE_ITEM_ERROR_MSG
    }
}
impl fmt::Display for ParseItemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", PARSE_ITEM_ERROR_MSG)
    }
}

fn str_to_vec(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
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
fn hms(timestamp: i64) -> String {
    let hours = timestamp / 60 / 60;
    let minutes = (timestamp - hours * 60) / 60;
    let seconds = timestamp - hours * 60 * 60 - minutes * 60;
    let mut res = "".to_string();
    if hours > 0 {
        res += &format!("{}h", hours);
    }
    if minutes > 0 {
        res += &format!("{}m", minutes);
    }
    if seconds > 0 {
        res += &format!("{}s", seconds);
    }
    res
}

// format timestamp
fn ft(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .unwrap()
        .format(DATE_FORMAT)
        .to_string()
}

// from the formatted string of fn ft, this parses the unix timestamp
fn tf(s: &str) -> Result<i64, ParseItemError> {
    let dt_timestamp = DateTime::parse_from_str(s, DATE_FORMAT)
        .ok()
        .ok_or(ParseItemError)?;
    return Ok(dt_timestamp.timestamp());
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.print_long(false))
    }
}

// removes prefix and suffix from raw metadata line
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
        lines.next(); // skip Table delimiter
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
            let pollished = pollish(raw_timetrack, PREFIX_TIMETRACK)?;
            if pollished.is_empty() {
                timetrack = vec![];
            } else {
                // Result implements FromIterator, so you can move the Result outside and iterators
                // will take care of the rest (including stopping iteration if an error is found).
                // https://stackoverflow.com/questions/26368288/how-do-i-stop-iteration-and-return-an-error-when-iteratormap-returns-a-result
                // super cool :D
                timetrack = pollished
                    .split(", ")
                    .map(|a| tf(a))
                    .collect::<Result<Vec<i64>, ParseItemError>>()?;
            }
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
        lines.next();
        let content = lines.map(|s| format!("{}\n", s)).collect();
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

    /**
     * short info about the item
     * prints only first line of the content and id
     * the id is surrounded by spaces to reach spacer_len
     */
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

    /// # Arguments
    /// * `serialize` - if the serialize flag is true, the timetrack is printed as date,
    /// else only the timedifferences are printed
    ///
    /// # Returns
    /// long info about the item
    /// first: a table of metadata
    /// second: content
    pub fn print_long(&self, serialize: bool) -> String {
        let tt;
        if serialize {
            tt = self
                .timetrack
                .iter()
                .map(|t| ft(*t))
                .collect::<Vec<String>>();
        } else {
            // calculate timedifferences
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
                .map(|t| hms(t))
                .collect::<Vec<String>>();
        }
        let res = format!(
            "{}{}|\n{}\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n\n{}",
            PREFIX_ID,
            self.id,
            TABLE_HEADER_DELIMITER,
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
