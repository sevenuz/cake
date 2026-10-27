use crate::error::ParseError;
use crate::util;
use core::fmt;
use serde_yaml_ng::{Mapping, Value};

#[cfg(test)]
mod tests;

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
    status: Option<Status>,
    tags: Vec<String>,
    parent: Option<String>,
    blocked_by: Vec<String>,
    timestamp: i64, // creation timestamp
    by: Option<String>, // creator, set once
    sessions: Vec<String>, // agent sessions that changed the item
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
        writeln!(f, "{}", self.print_long())
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
    match m.shift_remove(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(Value::Bool(b)) => Ok(Some(b.to_string())),
        Some(_) => Err(err(format!("'{}' must be a string", key))),
    }
}

fn take_list(m: &mut Mapping, key: &str) -> Result<Vec<String>, ParseError> {
    match m.shift_remove(key) {
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
            status: None,
            tags,
            parent,
            blocked_by: vec![],
            timestamp: util::timestamp(),
            by: None,
            sessions: vec![],
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
        m.shift_remove("kind"); // written by older versions, a status replaces it
        let status = take_str(&mut m, "status")?
            .map(|s| Status::parse(&s).map_err(err))
            .transpose()?;
        let tags = take_list(&mut m, "tags")?;
        let parent = take_str(&mut m, "parent")?.map(|p| unlink(&p));
        let blocked_by = take_list(&mut m, "blocked_by")?.iter().map(|s| unlink(s)).collect();
        let timestamp = take_time(&mut m, "created")?.unwrap_or(fallback_created);
        let by = take_str(&mut m, "by")?;
        let sessions = take_list(&mut m, "sessions")?;
        let closed = take_time(&mut m, "closed")?;
        let due = take_str(&mut m, "due")?;
        let priority = match m.shift_remove("priority") {
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
            status,
            tags,
            parent,
            blocked_by,
            timestamp,
            by,
            sessions,
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
        s += &format!("created: {}\n", util::format_timestamp(self.timestamp));
        if let Some(b) = &self.by {
            s += &format!("by: {}\n", yaml_str(b));
        }
        if !self.sessions.is_empty() {
            s += &format!("sessions: {}\n", list(&self.sessions, yaml_str));
        }
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

    /// one meta line (id, status, owner, dates, time, links) above the body
    pub fn print_long(&self) -> String {
        let mut meta = vec![
            format!("*{}*", self.id),
            self.status().map_or("note", |s| s.as_str()).to_string(),
        ];
        if !self.tags.is_empty() {
            meta.push(self.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "));
        }
        if let Some(b) = &self.by {
            meta.push(format!("by {}", b));
        }
        meta.push(format!("created {}", util::format_timestamp(self.timestamp)[..16].replace('T', " ")));
        if let Some(c) = self.closed {
            meta.push(format!("closed {}", util::format_timestamp(c)[..16].replace('T', " ")));
        }
        let tracked = self.tracked_seconds();
        if tracked > 0 {
            let running = if self.is_started() { " running" } else { "" };
            meta.push(format!("tracked {}{}", util::timestamp_to_hms(tracked), running));
        }
        if let Some(p) = self.priority {
            meta.push(format!("prio {}", p));
        }
        if let Some(d) = &self.due {
            meta.push(format!("due {}", d));
        }
        if let Some(p) = &self.parent {
            meta.push(format!("parent {}", p));
        }
        if !self.blocked_by.is_empty() {
            meta.push(format!("blocked by {}", self.blocked_by.join(", ")));
        }
        format!("{}\n\n{}", meta.join(" · "), self.content)
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

    /// seconds the running timer runs, None if stopped
    pub fn running_seconds(&self) -> Option<i64> {
        self.is_started()
            .then(|| util::timestamp() - self.timetrack.last().unwrap())
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
            self.status = Some(Status::Doing);
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

    pub fn blocked_by(&self) -> &Vec<String> {
        &self.blocked_by
    }

    pub fn by(&self) -> Option<&String> {
        self.by.as_ref()
    }

    pub fn add_session(&mut self, session: &str) {
        if !self.sessions.iter().any(|s| s == session) {
            self.sessions.push(session.to_string());
        }
    }

    pub fn set_by(&mut self, by: String) {
        self.by = Some(by);
    }

    pub fn priority(&self) -> Option<u64> {
        self.priority
    }

    pub fn due(&self) -> Option<&String> {
        self.due.as_ref()
    }

    /// todo or doing
    pub fn is_open(&self) -> bool {
        self.status.is_some_and(|s| s.is_open())
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    /// sets the status, also turns a note into a task: done/dropped stop a
    /// running timer and set `closed`, todo/doing remove it
    pub fn set_status(&mut self, status: Status) -> Result<(), String> {
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

    /// `due` is a date (2026-10-30) or an RFC 3339 datetime
    pub fn set_due(&mut self, due: Option<String>) -> Result<(), String> {
        if let Some(d) = &due {
            if chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err()
                && util::parse_timestamp(d).is_err()
            {
                return Err(format!("invalid due '{}', use 2026-10-30 or an RFC 3339 time", d));
            }
        }
        self.due = due;
        Ok(())
    }

    /// 1 (high) .. 3 (low)
    pub fn set_priority(&mut self, priority: Option<u64>) -> Result<(), String> {
        if priority.is_some_and(|p| !(1..=3).contains(&p)) {
            return Err("priority must be 1 (high) .. 3 (low)".to_string());
        }
        self.priority = priority;
        Ok(())
    }

    pub fn set_blocked_by(&mut self, ids: Vec<String>) {
        self.blocked_by = ids;
    }

    /// first `# heading`, else the first non-empty line
    pub fn title(&self) -> String {
        let lines = || self.content.lines().map(|l| l.trim());
        lines()
            .find_map(|l| l.strip_prefix("# "))
            .or_else(|| lines().find(|l| !l.is_empty()))
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    /// tracked time, a running interval counts until now
    pub fn tracked_seconds(&self) -> i64 {
        self.timetrack
            .chunks(2)
            .map(|p| p.get(1).copied().unwrap_or_else(util::timestamp) - p[0])
            .sum()
    }

    /// stored fields plus title, body and time stats; derived relations are added by the store
    pub fn to_json(&self) -> serde_json::Map<String, serde_json::Value> {
        let time: Vec<_> = self
            .timetrack
            .chunks(2)
            .map(|p| {
                serde_json::json!({
                    "start": util::format_timestamp(p[0]),
                    "end": p.get(1).map(|t| util::format_timestamp(*t)),
                })
            })
            .collect();
        let json = serde_json::json!({
            "id": self.id,
            "status": self.status.map(|s| s.as_str()),
            "tags": self.tags,
            "parent": self.parent,
            "blocked_by": self.blocked_by,
            "created": util::format_timestamp(self.timestamp),
            "by": self.by,
            "sessions": self.sessions,
            "closed": self.closed.map(util::format_timestamp),
            "due": self.due,
            "priority": self.priority,
            "time": time,
            "tracked_seconds": self.tracked_seconds(),
            "running": self.is_started(),
            "extra": serde_json::to_value(&self.extra).unwrap_or_default(),
            "title": self.title(),
            "body": self.content,
            "children": self.children,
        });
        match json {
            serde_json::Value::Object(m) => m,
            _ => unreachable!(),
        }
    }

    /// replaces the item but keeps the derived children
    pub fn set(&mut self, item: Item) {
        let children = std::mem::take(&mut self.children);
        *self = item;
        self.children = children;
    }
}
