use crate::error::ParseError;
use crate::util;
use core::fmt;
use serde_yaml_ng::{Mapping, Value};
use termimad::crossterm::style::Stylize;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Task,
    Note,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    Todo,
    Doing,
    Done,
    Dropped,
}

impl Status {
    pub fn parse(s: &str) -> Result<Status, String> {
        match s {
            "todo" => Ok(Status::Todo),
            "doing" => Ok(Status::Doing),
            "done" => Ok(Status::Done),
            "dropped" => Ok(Status::Dropped),
            _ => Err(format!("unknown status '{}', use todo|doing|done|dropped", s)),
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Status::Todo | Status::Doing)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::Doing => "doing",
            Status::Done => "done",
            Status::Dropped => "dropped",
        }
    }
}

#[derive(Clone)]
pub struct Item {
    id: String,
    kind: Kind,
    status: Option<Status>,
    tags: Vec<String>,
    parent: Option<String>,
    blocked_by: Vec<String>,
    related: Vec<String>,
    timestamp: i64, // creation timestamp
    closed: Option<i64>,
    due: Option<String>,
    priority: Option<u64>,
    timetrack: Vec<i64>,
    extra: Mapping,
    content: String,
    children: Vec<String>, // derived by store
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.print_long(false))
    }
}

fn err(message: String) -> ParseError {
    ParseError { message }
}

fn yaml_str(s: &str) -> String {
    if !s.is_empty()
        && s.chars().all(|c| c.is_alphanumeric() || "_-./".contains(c))
        && !s.starts_with(['-', '.'])
    {
        s.to_string()
    } else {
        serde_json::to_string(s).unwrap()
    }
}

fn link(id: &str) -> String {
    serde_json::to_string(&format!("[[{}]]", id)).unwrap()
}

fn unlink(s: &str) -> String {
    s.trim()
        .trim_start_matches("[[")
        .trim_end_matches("]]")
        .to_string()
}

fn list(v: &[String], f: fn(&str) -> String) -> String {
    format!("[{}]", v.iter().map(|s| f(s)).collect::<Vec<_>>().join(", "))
}

fn take_str(m: &mut Mapping, key: &str) -> Result<Option<String>, ParseError> {
    match m.remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(Value::Bool(b)) => Ok(Some(b.to_string())),
        Some(_) => Err(err(format!("'{}' must be a string", key))),
    }
}

fn take_list(m: &mut Mapping, key: &str) -> Result<Vec<String>, ParseError> {
    match m.remove(key) {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::Sequence(seq)) => seq
            .into_iter()
            .map(|v| match v {
                Value::String(s) => Ok(s),
                Value::Number(n) => Ok(n.to_string()),
                _ => Err(err(format!("'{}' must be a list of strings", key))),
            })
            .collect(),
        Some(Value::String(s)) => Ok(util::split_comma_cleanup(s)),
        Some(_) => Err(err(format!("'{}' must be a list", key))),
    }
}

fn take_time(m: &mut Mapping, key: &str) -> Result<Option<i64>, ParseError> {
    take_str(m, key)?
        .map(|s| util::parse_timestamp(&s))
        .transpose()
}

impl Item {
    pub fn new(id: String, children: Vec<String>, parent: Option<String>, tags: Vec<String>) -> Self {
        Self {
            id,
            kind: Kind::Task,
            status: Some(Status::Todo),
            tags,
            parent,
            blocked_by: vec![],
            related: vec![],
            timestamp: util::timestamp(),
            closed: None,
            due: None,
            priority: None,
            timetrack: vec![],
            extra: Mapping::new(),
            content: String::new(),
            children,
        }
    }

    /// lenient parsing: missing fields get defaults, malformed fields are errors
    pub fn from_markdown(s: &str, fallback_id: &str, fallback_created: i64) -> Result<Self, ParseError> {
        let s = s.replace("\r\n", "\n");
        let (front, content) = match s.strip_prefix("---\n").and_then(|r| {
            r.find("\n---\n")
                .map(|i| (&r[..i], &r[i + 5..]))
                .or_else(|| r.strip_suffix("\n---").map(|f| (f, "")))
        }) {
            Some((f, c)) => (f.to_string(), c.to_string()),
            None => (String::new(), s.clone()),
        };
        let mut m: Mapping = if front.trim().is_empty() {
            Mapping::new()
        } else {
            serde_yaml_ng::from_str(&front).map_err(|e| err(e.to_string()))?
        };
        let id = take_str(&mut m, "id")?.unwrap_or(fallback_id.to_string());
        let kind = match take_str(&mut m, "kind")?.as_deref() {
            None | Some("task") => Kind::Task,
            Some("note") => Kind::Note,
            Some(k) => return Err(err(format!("unknown kind '{}'", k))),
        };
        let status = match take_str(&mut m, "status")?.as_deref() {
            None if kind == Kind::Task => Some(Status::Todo),
            None => None,
            Some(s) => Some(Status::parse(s).map_err(err)?),
        };
        let tags = take_list(&mut m, "tags")?;
        let parent = take_str(&mut m, "parent")?.map(|p| unlink(&p));
        let blocked_by = take_list(&mut m, "blocked_by")?.iter().map(|s| unlink(s)).collect();
        let related = take_list(&mut m, "related")?.iter().map(|s| unlink(s)).collect();
        let timestamp = take_time(&mut m, "created")?.unwrap_or(fallback_created);
        let closed = take_time(&mut m, "closed")?;
        let due = take_str(&mut m, "due")?;
        let priority = match m.remove("priority") {
            None | Some(Value::Null) => None,
            Some(Value::Number(n)) if n.as_u64().is_some() => n.as_u64(),
            Some(_) => return Err(err("'priority' must be a number".to_string())),
        };
        let mut timetrack = vec![];
        for interval in take_list(&mut m, "time")? {
            let (start, end) = interval
                .split_once('/')
                .ok_or(err(format!("invalid interval '{}'", interval)))?;
            timetrack.push(util::parse_timestamp(start)?);
            if !end.trim().is_empty() {
                timetrack.push(util::parse_timestamp(end)?);
            } else if timetrack.len() % 2 == 1 {
                break;
            }
        }
        Ok(Item {
            id,
            kind,
            status,
            tags,
            parent,
            blocked_by,
            related,
            timestamp,
            closed,
            due,
            priority,
            timetrack,
            extra: m,
            content,
            children: vec![],
        })
    }

    pub fn to_markdown(&self) -> String {
        let mut s = format!("---\nid: {}\n", yaml_str(&self.id));
        if self.kind == Kind::Note {
            s += "kind: note\n";
        }
        if let Some(st) = self.status {
            s += &format!("status: {}\n", st.as_str());
        }
        if !self.tags.is_empty() {
            s += &format!("tags: {}\n", list(&self.tags, yaml_str));
        }
        if let Some(p) = &self.parent {
            s += &format!("parent: {}\n", link(p));
        }
        if !self.blocked_by.is_empty() {
            s += &format!("blocked_by: {}\n", list(&self.blocked_by, link));
        }
        if !self.related.is_empty() {
            s += &format!("related: {}\n", list(&self.related, link));
        }
        s += &format!("created: {}\n", util::format_timestamp(self.timestamp));
        if let Some(c) = self.closed {
            s += &format!("closed: {}\n", util::format_timestamp(c));
        }
        if let Some(d) = &self.due {
            s += &format!("due: {}\n", yaml_str(d));
        }
        if let Some(p) = self.priority {
            s += &format!("priority: {}\n", p);
        }
        if !self.timetrack.is_empty() {
            s += "time:\n";
            for pair in self.timetrack.chunks(2) {
                s += &format!(
                    "  - {}/{}\n",
                    util::format_timestamp(pair[0]),
                    pair.get(1).map(|t| util::format_timestamp(*t)).unwrap_or_default()
                );
            }
        }
        if !self.extra.is_empty() {
            s += &serde_yaml_ng::to_string(&self.extra).unwrap();
        }
        s + "---\n" + &self.content
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
    /// * `serialize` - if true, the timetrack is printed as date,
    /// else only the timedifferences are printed
    pub fn print_long(&self, serialize: bool) -> String {
        let tt = if serialize {
            self.timetrack
                .iter()
                .map(|t| util::format_timestamp(*t))
                .collect::<Vec<String>>()
        } else {
            self.timetrack
                .chunks(2)
                .filter(|p| p.len() == 2)
                .map(|p| util::timestamp_to_hms(p[1] - p[0]))
                .collect::<Vec<String>>()
        };
        format!(
            "| id | {}|\n|---|---|\n| status | {}|\n| created | {}|\n| tags | {}|\n| timetrack | {}|\n| parent | {}|\n| children | {}|\n| blocked by | {}|\n\n{}",
            self.id,
            self.status().map(|s| s.as_str()).unwrap_or("note"),
            util::format_timestamp(self.timestamp),
            util::vec_to_str(&self.tags),
            util::vec_to_str(&tt),
            self.parent.clone().unwrap_or_default(),
            util::vec_to_str(&self.children),
            util::vec_to_str(&self.blocked_by),
            self.content
        )
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn children(&self) -> &Vec<String> {
        &self.children
    }

    pub fn add_child(&mut self, child: &str) {
        if !self.children.iter().any(|c| c == child) {
            self.children.push(child.to_owned());
        }
    }

    pub fn retain_child(&mut self, child: &str) {
        self.children.retain(|s| s != child);
    }

    pub fn parents(&self) -> &[String] {
        self.parent.as_slice()
    }

    pub fn set_parent(&mut self, parent: Option<String>) {
        self.parent = parent;
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn append_tags(&mut self, tags: Vec<String>) -> &mut Self {
        for tag in tags {
            if !self.tags.contains(&tag) {
                self.tags.push(tag);
            }
        }
        self
    }

    pub fn remove_tags(&mut self, tags: Vec<String>) {
        self.tags.retain(|tag| !tags.contains(tag));
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn is_started(&self) -> bool {
        self.timetrack.len() % 2 == 1
    }

    pub fn is_stopped(&self) -> bool {
        self.timetrack.len() % 2 == 0
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_stopped() {
            self.timetrack.push(util::timestamp());
            if self.kind == Kind::Task {
                self.status = Some(Status::Doing);
            }
            Ok(())
        } else {
            Err(format!(
                "{} runs already since {}",
                self.id,
                util::format_timestamp(*self.timetrack.last().unwrap())
            ))
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.is_started() {
            self.timetrack.push(util::timestamp());
            Ok(())
        } else {
            Err(format!("{} is not running.", self.id))
        }
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    /// sets the task status: done/dropped stop a running timer and set
    /// `closed`, todo/doing remove it
    pub fn set_status(&mut self, status: Status) -> Result<(), String> {
        if self.kind == Kind::Note {
            return Err(format!("{} is a note, notes have no status.", self.id));
        }
        if status.is_open() {
            self.closed = None;
        } else {
            if self.is_started() {
                self.stop()?;
            }
            self.closed = Some(util::timestamp());
        }
        self.status = Some(status);
        Ok(())
    }

    /// replaces the item but keeps the derived children
    pub fn set(&mut self, item: Item) {
        let children = std::mem::take(&mut self.children);
        *self = item;
        self.children = children;
    }
}
