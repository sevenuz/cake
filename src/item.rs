use std::collections::BTreeMap;
use crate::error::ParseError;
use crate::util;
use core::fmt;
use serde_yaml_ng::{Mapping, Value};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    /// new, not sorted yet
    Inbox,
    /// to do, has an assignee
    Task,
    Doing,
    /// finished, the owner checks it
    Review,
    Done,
    Dropped,
}

impl Status {
    /// a status name; `note` (or none) is no status, todo is the old task
    pub fn parse(s: &str) -> Result<Option<Status>, String> {
        match s {
            "note" | "none" => Ok(None),
            "inbox" => Ok(Some(Status::Inbox)),
            "task" | "todo" => Ok(Some(Status::Task)),
            "doing" => Ok(Some(Status::Doing)),
            "review" => Ok(Some(Status::Review)),
            "done" => Ok(Some(Status::Done)),
            "dropped" => Ok(Some(Status::Dropped)),
            _ => Err(format!("unknown status '{}', use inbox|note|task|doing|review|done|dropped", s)),
        }
    }

    /// not closed: still needs someone (blocks, counts in the score)
    pub fn is_open(&self) -> bool {
        matches!(self, Status::Inbox | Status::Task | Status::Doing | Status::Review)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Inbox => "inbox",
            Status::Task => "task",
            Status::Review => "review",
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
    usage: BTreeMap<String, Usage>, // tokens by model
    closed: Option<i64>,
    due: Option<String>,
    deadline: Option<String>,
    assignee: Option<String>,
    owner: Option<String>,
    priority: Option<u64>,
    /// expected work, seconds
    estimate: Option<i64>,
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
            usage: BTreeMap::new(),
            closed: None,
            due: None,
            deadline: None,
            assignee: None,
            owner: None,
            priority: None,
            estimate: None,
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
            .transpose()?
            .flatten();
        let assignee = take_str(&mut m, "assignee")?;
        let owner = take_str(&mut m, "owner")?;
        let tags = take_list(&mut m, "tags")?;
        let parent = take_str(&mut m, "parent")?.map(|p| unlink(&p));
        let blocked_by = take_list(&mut m, "blocked_by")?.iter().map(|s| unlink(s)).collect();
        let timestamp = take_time(&mut m, "created")?.unwrap_or(fallback_created);
        let by = take_str(&mut m, "by")?;
        let sessions = take_list(&mut m, "sessions")?;
        let usage = match m.shift_remove("usage") {
            None | Some(Value::Null) => BTreeMap::new(),
            Some(v) => serde_yaml_ng::from_value(v).map_err(|e| err(format!("'usage': {}", e)))?,
        };
        let closed = take_time(&mut m, "closed")?;
        let due = take_str(&mut m, "due")?;
        let deadline = take_str(&mut m, "deadline")?;
        let priority = match m.shift_remove("priority") {
            None | Some(Value::Null) => None,
            Some(Value::Number(n)) if n.as_u64().is_some() => n.as_u64(),
            Some(_) => return Err(err("'priority' must be a number".to_string())),
        };
        let estimate = match take_str(&mut m, "estimate")? {
            None => None,
            Some(e) => Some(util::parse_duration(&e).map_err(|_| err(format!("'estimate' {} is no duration like 2h30m", e)))?),
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
            usage,
            closed,
            due,
            deadline,
            assignee,
            owner,
            priority,
            estimate,
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
        if !self.usage.is_empty() {
            let models: Vec<String> = self
                .usage
                .iter()
                .map(|(m, u)| {
                    format!(
                        "{}: {{in: {}, out: {}, cache_read: {}, cache_write: {}}}",
                        yaml_str(m),
                        u.input,
                        u.output,
                        u.cache_read,
                        u.cache_write
                    )
                })
                .collect();
            s += &format!("usage: {{{}}}\n", models.join(", "));
        }
        if let Some(c) = self.closed {
            s += &format!("closed: {}\n", util::format_timestamp(c));
        }
        if let Some(d) = &self.due {
            s += &format!("due: {}\n", yaml_str(d));
        }
        if let Some(d) = &self.deadline {
            s += &format!("deadline: {}\n", yaml_str(d));
        }
        if let Some(a) = &self.assignee {
            s += &format!("assignee: {}\n", yaml_str(a));
        }
        if let Some(o) = &self.owner {
            s += &format!("owner: {}\n", yaml_str(o));
        }
        if let Some(p) = self.priority {
            s += &format!("priority: {}\n", p);
        }
        if let Some(e) = self.estimate {
            s += &format!("estimate: {}\n", util::timestamp_to_hms(e));
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
        if !self.usage.is_empty() {
            meta.push(crate::footprint::describe(&self.usage));
        }
        if let Some(p) = self.priority {
            meta.push(format!("prio {}", p));
        }
        if let Some(e) = self.estimate {
            meta.push(format!("est {}", util::timestamp_to_hms(e)));
        }
        if let Some(d) = &self.due {
            meta.push(format!("due {}", d));
        }
        if let Some(d) = &self.deadline {
            meta.push(format!("deadline {}", d));
        }
        if let Some(a) = &self.assignee {
            meta.push(format!("for {}", a));
        }
        if let Some(o) = &self.owner {
            meta.push(format!("owner {}", o));
        }
        if let Some(p) = &self.parent {
            meta.push(format!("parent {}", p));
        }
        if !self.blocked_by.is_empty() {
            meta.push(format!("blocked by {}", self.blocked_by.join(", ")));
        }
        format!("{}\n\n{}", meta.join(crate::style::g().sep), self.content)
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    /// a new id; the store fixes everything that points here (Store::rename)
    pub fn set_id(&mut self, id: String) {
        self.id = id;
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

    pub fn closed(&self) -> Option<i64> {
        self.closed
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// tracked intervals as (start, end), a running one ends now
    pub fn add_usage(&mut self, model: &str, u: Usage) {
        *self.usage.entry(model.to_string()).or_default() += u;
    }

    pub fn sessions(&self) -> &Vec<String> {
        &self.sessions
    }

    pub fn intervals(&self) -> Vec<(i64, i64)> {
        self.timetrack
            .chunks(2)
            .map(|p| (p[0], p.get(1).copied().unwrap_or_else(util::timestamp)))
            .collect()
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

    pub fn deadline(&self) -> Option<&String> {
        self.deadline.as_ref()
    }

    pub fn assignee(&self) -> Option<&String> {
        self.assignee.as_ref()
    }

    pub fn set_assignee(&mut self, a: Option<String>) {
        self.assignee = a;
    }

    pub fn owner(&self) -> Option<&String> {
        self.owner.as_ref()
    }

    pub fn set_owner(&mut self, o: Option<String>) {
        self.owner = o;
    }

    /// back to a plain note: no status
    pub fn set_note(&mut self) -> Result<(), String> {
        if self.is_started() {
            self.stop()?;
        }
        self.status = None;
        self.closed = None;
        Ok(())
    }

    /// todo or doing
    pub fn is_open(&self) -> bool {
        self.status.is_some_and(|s| s.is_open())
    }

    pub fn status(&self) -> Option<Status> {
        self.status
    }

    /// sets the status: review, done and dropped stop a running timer,
    /// done and dropped set `closed`
    pub fn set_status(&mut self, status: Status) -> Result<(), String> {
        // the timer only runs while someone works on it
        if self.is_started() && !matches!(status, Status::Task | Status::Doing) {
            self.stop()?;
        }
        self.closed = (!status.is_open()).then(util::timestamp);
        self.status = Some(status);
        Ok(())
    }

    /// soft target: a date in any form of util::parse_date, stored as YYYY-MM-DD
    pub fn set_due(&mut self, due: Option<String>) -> Result<(), String> {
        self.due = due.map(|d| util::parse_date(&d).map_err(|e| format!("due: {}", e))).transpose()?;
        Ok(())
    }

    /// hard limit, same forms as `due`
    #[cfg(test)]
    pub fn set_timetrack(&mut self, t: Vec<i64>) {
        self.timetrack = t;
    }

    pub fn estimate(&self) -> Option<i64> {
        self.estimate
    }

    /// a duration like 2h30m, 45m, 1d
    pub fn set_estimate(&mut self, estimate: Option<String>) -> Result<(), String> {
        self.estimate = match estimate {
            None => None,
            Some(e) => match util::parse_duration(&e) {
                Ok(s) if s > 0 => Some(s),
                _ => return Err(format!("estimate: '{}' is no duration like 2h30m", e)),
            },
        };
        Ok(())
    }

    pub fn set_deadline(&mut self, deadline: Option<String>) -> Result<(), String> {
        self.deadline = deadline.map(|d| util::parse_date(&d).map_err(|e| format!("deadline: {}", e))).transpose()?;
        Ok(())
    }

    /// appends `text` as a new paragraph, e.g. the answer to a question
    pub fn append_content(&mut self, text: &str) {
        let body = self.content.trim_end();
        self.content = format!("{}\n\n{}\n", body, text.trim());
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
        title_of(&self.content)
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
            "usage": self.usage,
            "footprint": crate::footprint::estimate(&self.usage),
            "closed": self.closed.map(util::format_timestamp),
            "due": self.due,
            "deadline": self.deadline,
            "assignee": self.assignee,
            "owner": self.owner,
            "priority": self.priority,
            "estimate": self.estimate,
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

/// the first `# heading`, else the first non-empty line
pub fn title_of(content: &str) -> String {
    let lines = || content.lines().map(|l| l.trim());
    lines()
        .find_map(|l| l.strip_prefix("# "))
        .or_else(|| lines().find(|l| !l.is_empty()))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// tokens of one model
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Usage {
    #[serde(rename = "in", default)]
    pub input: u64,
    #[serde(rename = "out", default)]
    pub output: u64,
    #[serde(default)]
    pub cache_read: u64,
    #[serde(default)]
    pub cache_write: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }

    /// `self` split into `n` parts, the remainder in the first
    pub fn split(&self, n: u64, first: bool) -> Usage {
        let part = |v: u64| v / n + if first { v % n } else { 0 };
        Usage {
            input: part(self.input),
            output: part(self.output),
            cache_read: part(self.cache_read),
            cache_write: part(self.cache_write),
        }
    }
}

impl std::ops::AddAssign for Usage {
    fn add_assign(&mut self, o: Usage) {
        self.input += o.input;
        self.output += o.output;
        self.cache_read += o.cache_read;
        self.cache_write += o.cache_write;
    }
}
