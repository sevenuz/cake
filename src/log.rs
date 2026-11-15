//! the session log: who was active in a store and when, also without a
//! timer. One file per month and machine, `<store>/log/YYYY-MM-<host>.jsonl`,
//! so two machines never write the same file and sync never conflicts.
//! Lines are only appended. `kind` is `chat` (a Claude Code session, from
//! the Stop hook) or `cake` (the form or dashboard open); `item` is set
//! while the session had a task with a running timer.

use crate::util;
use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};

/// a pause longer than this ends a stretch of activity
pub const GAP: i64 = 15 * 60;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Line {
    /// RFC 3339
    pub start: String,
    pub end: String,
    pub who: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
}

impl Line {
    pub fn new(start: i64, end: i64, who: &str, kind: &str) -> Line {
        Line {
            start: util::format_timestamp(start),
            end: util::format_timestamp(end),
            who: who.to_string(),
            kind: kind.to_string(),
            agent: None,
            session: None,
            item: None,
        }
    }

    /// (start, end) as unix seconds, None if unreadable
    pub fn span(&self) -> Option<(i64, i64)> {
        Some((util::parse_timestamp(&self.start).ok()?, util::parse_timestamp(&self.end).ok()?))
    }
}

/// this machine's name for the file name, `host` if unknown
pub fn host() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: the buffer is valid for its length; gethostname terminates
    // the name with a 0 byte when it fits
    let ok = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) } == 0;
    let name: String = if ok {
        String::from_utf8_lossy(&buf[..buf.iter().position(|b| *b == 0).unwrap_or(0)]).to_string()
    } else {
        String::new()
    };
    let name: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    if name.is_empty() { "host".into() } else { name }
}

fn dir(root: &Path) -> PathBuf {
    root.join("log")
}

/// the file of the month `at` falls in, on this machine
pub fn file(root: &Path, at: i64) -> PathBuf {
    let month = Local.timestamp_opt(at, 0).single().map_or("0000-00".into(), |t| t.format("%Y-%m").to_string());
    dir(root).join(format!("{}-{}.jsonl", month, host()))
}

/// appends `lines` (each to the file of its start month); returns the files
pub fn append(root: &Path, lines: &[Line]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<PathBuf> = vec![];
    for line in lines {
        let Some((start, _)) = line.span() else { continue };
        let path = file(root, start);
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path)?;
        writeln!(f, "{}", serde_json::to_string(line)?)?;
        if !files.contains(&path) {
            files.push(path);
        }
    }
    Ok(files)
}

/// every line of every machine that overlaps [from, to]
pub fn read(root: &Path, from: i64, to: i64) -> Vec<Line> {
    let Ok(entries) = std::fs::read_dir(dir(root)) else { return vec![] };
    let mut out = vec![];
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().map_or(true, |x| x != "jsonl") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for l in text.lines() {
            let Ok(line) = serde_json::from_str::<Line>(l) else { continue };
            if line.span().is_some_and(|(s, e)| s <= to && e >= from) {
                out.push(line);
            }
        }
    }
    out.sort_by(|a, b| a.start.cmp(&b.start));
    out
}

/// activity stretches from moments of activity (sorted or not): moments
/// closer than GAP join; `after` is where the last stretch ended, a first
/// moment within GAP of it continues from there (the pause in between counts)
pub fn stretches(times: &[i64], after: Option<i64>) -> Vec<(i64, i64)> {
    let mut times = times.to_vec();
    times.sort();
    let mut out: Vec<(i64, i64)> = vec![];
    for t in times {
        match out.last_mut() {
            Some((_, e)) if t - *e <= GAP => *e = (*e).max(t),
            _ => match after {
                Some(a) if out.is_empty() && t >= a && t - a <= GAP => out.push((a, t)),
                _ => out.push((t, t)),
            },
        }
    }
    out.retain(|(s, e)| e > s);
    out
}

/// splits [start, end] by the item timers `timers` (id, start, end): the
/// parts with a timer get its id (the first that runs), the rest None;
/// neighbours with the same id join
pub fn split(start: i64, end: i64, timers: &[(String, i64, i64)]) -> Vec<(i64, i64, Option<String>)> {
    let mut cuts = vec![start, end];
    for (_, s, e) in timers {
        cuts.extend([*s, *e].into_iter().filter(|c| *c > start && *c < end));
    }
    cuts.sort();
    cuts.dedup();
    let mut out: Vec<(i64, i64, Option<String>)> = vec![];
    for w in cuts.windows(2) {
        let mid = w[0] + (w[1] - w[0]) / 2;
        let id = timers.iter().find(|(_, s, e)| *s <= mid && mid < *e).map(|(id, _, _)| id.clone());
        match out.last_mut() {
            Some(last) if last.2 == id => last.1 = w[1],
            _ => out.push((w[0], w[1], id)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stretches_join_close_moments_and_continue_after_the_last() {
        let m = 60;
        assert_eq!(stretches(&[0, 5 * m, 10 * m, 40 * m, 41 * m], None), vec![(0, 10 * m), (40 * m, 41 * m)]);
        // a single moment is no stretch
        assert_eq!(stretches(&[0], None), vec![]);
        // the last stretch ended at 0: 10 min later continues it, 20 does not
        assert_eq!(stretches(&[10 * m, 12 * m], Some(0)), vec![(0, 12 * m)]);
        assert_eq!(stretches(&[20 * m, 22 * m], Some(0)), vec![(20 * m, 22 * m)]);
    }

    #[test]
    fn split_gives_timer_parts_their_item() {
        let t = vec![("a".to_string(), 10, 20), ("b".to_string(), 30, 50)];
        assert_eq!(
            split(0, 40, &t),
            vec![(0, 10, None), (10, 20, Some("a".into())), (20, 30, None), (30, 40, Some("b".into()))]
        );
        assert_eq!(split(12, 18, &t), vec![(12, 18, Some("a".into()))]);
    }

    #[test]
    fn lines_go_to_the_month_file_and_read_back_by_overlap() {
        let dir = tempfile::tempdir().unwrap();
        let start = util::parse_timestamp("2026-11-15T09:00:00+01:00").unwrap();
        let mut l = Line::new(start, start + 3600, "jul", "chat");
        l.item = Some("x".into());
        let files = append(dir.path(), &[l.clone(), Line::new(start + 7200, start + 7300, "jul", "cake")]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy().starts_with("2026-11-"));
        assert_eq!(read(dir.path(), start + 1800, start + 1900), vec![l]);
        assert_eq!(read(dir.path(), start, start + 8000).len(), 2);
    }
}
