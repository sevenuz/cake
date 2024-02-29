use crate::error;
use crate::util;
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

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.print_long(false))
    }
}

// https://doc.rust-lang.org/std/str/trait.FromStr.html
impl FromStr for Item {
    type Err = error::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = error::ParseError {
            message: "Invalid string to create an item".to_string(),
        };
        let mut lines = s.lines();
        let id: String;
        if let Some(raw_id) = lines.next() {
            id = util::extract_metadata(raw_id, PREFIX_ID)?;
        } else {
            return Err(err);
        };
        lines.next(); // skip Table delimiter
        let timestamp: i64;
        if let Some(raw_timestamp) = lines.next() {
            timestamp =
                util::parse_timestamp(&util::extract_metadata(raw_timestamp, PREFIX_TIMESTAMP)?)?;
        } else {
            return Err(err);
        };
        let last_modified: i64;
        if let Some(raw_last_modified) = lines.next() {
            last_modified = util::parse_timestamp(&util::extract_metadata(
                raw_last_modified,
                PREFIX_LAST_MODIFIED,
            )?)?;
        } else {
            return Err(err);
        };
        let tags: Vec<String>;
        if let Some(raw_tags) = lines.next() {
            tags = util::str_to_vec(&util::extract_metadata(raw_tags, PREFIX_TAGS)?);
        } else {
            return Err(err);
        };
        let timetrack: Vec<i64>;
        if let Some(raw_timetrack) = lines.next() {
            let pollished = util::extract_metadata(raw_timetrack, PREFIX_TIMETRACK)?;
            if pollished.is_empty() {
                timetrack = vec![];
            } else {
                // Result implements FromIterator, so you can move the Result outside and iterators
                // will take care of the rest (including stopping iteration if an error is found).
                // https://stackoverflow.com/questions/26368288/how-do-i-stop-iteration-and-return-an-error-when-iteratormap-returns-a-result
                // super cool :D
                timetrack = pollished
                    .split(", ")
                    .map(|a| util::parse_timestamp(a))
                    .collect::<Result<Vec<i64>, error::ParseError>>()?;
            }
        } else {
            return Err(err);
        };
        let parents: Vec<String>;
        if let Some(raw_parents) = lines.next() {
            parents = util::str_to_vec(&util::extract_metadata(raw_parents, PREFIX_PARENTS)?);
        } else {
            return Err(err);
        };
        let children: Vec<String>;
        if let Some(raw_children) = lines.next() {
            children = util::str_to_vec(&util::extract_metadata(raw_children, PREFIX_CHILDREN)?);
        } else {
            return Err(err);
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
                .map(|t| util::format_timestamp(*t))
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
                .map(|t| util::timestamp_to_hms(t))
                .collect::<Vec<String>>();
        }
        let res = format!(
            "{}{}|\n{}\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n{}{}|\n\n{}",
            PREFIX_ID,
            self.id,
            TABLE_HEADER_DELIMITER,
            PREFIX_TIMESTAMP,
            util::format_timestamp(self.timestamp),
            PREFIX_LAST_MODIFIED,
            util::format_timestamp(self.last_modified),
            PREFIX_TAGS,
            util::vec_to_str(&self.tags),
            PREFIX_TIMETRACK,
            util::vec_to_str(&tt),
            PREFIX_PARENTS,
            util::vec_to_str(&self.parents),
            PREFIX_CHILDREN,
            util::vec_to_str(&self.children),
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

    pub fn append_tags(&mut self, tags: Vec<String>) -> &mut Self {
        for tag in tags {
            if !self.tags.contains(&tag) {
                self.tags.push(tag.to_owned());
            }
        }
        self.update_last_modified();
        self
    }

    pub fn remove_tags(&mut self, tags: Vec<String>) {
        self.tags = self
            .tags
            .iter()
            .filter(|tag| !tags.contains(tag))
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
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
                util::format_timestamp(*self.timetrack.last().unwrap())
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
