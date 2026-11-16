//! tokens per item. Agents report them with `cake usage add`; for Claude
//! Code, `cake usage claude-hook` (a Stop hook) reads the new lines of the
//! session transcript and gives each answer to the items of that session
//! whose timer ran at that moment. Stopped items get it right away, running
//! ones collect it in a local pending file until their timer stops.

use crate::item::Usage;
use crate::log;
use crate::store::Store;
use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// local state, not in git: ~/.local/state/cake
fn state_dir() -> Result<PathBuf, Box<dyn Error>> {
    let base = directories::BaseDirs::new().ok_or("no home directory")?;
    let dir = base.state_dir().unwrap_or(base.data_local_dir()).join("cake");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// item -> model -> usage, waiting for the item's timer to stop
type Pending = BTreeMap<String, BTreeMap<String, Usage>>;

fn pending_file(root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let abs = std::fs::canonicalize(root).unwrap_or(root.to_path_buf());
    let name = abs.to_string_lossy().replace('/', "%");
    Ok(state_dir()?.join(format!("pending{}.json", name)))
}

fn read_json<T: for<'a> Deserialize<'a> + Default>(path: &Path) -> T {
    std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn write_json<T: Serialize>(path: &Path, v: &T) -> Result<(), Box<dyn Error>> {
    std::fs::write(path, serde_json::to_string(v)?)?;
    Ok(())
}

/// moves pending usage of stopped items into them; true if any moved
pub fn fold(store: &mut Store) -> Result<bool, Box<dyn Error>> {
    let file = pending_file(store.root())?;
    let mut pending: Pending = read_json(&file);
    if pending.is_empty() {
        return Ok(false);
    }
    let ready: Vec<String> = pending
        .keys()
        .filter(|id| store.get_item(id).map_or(true, |i| !i.is_started()))
        .cloned()
        .collect();
    for id in &ready {
        let models = pending.remove(id).unwrap();
        if let Some(item) = store.get_item_mut(id) {
            for (model, u) in models {
                item.add_usage(&model, u);
            }
        }
    }
    write_json(&file, &pending)?;
    Ok(!ready.is_empty())
}

/// what Claude Code sends a Stop hook on stdin
#[derive(Deserialize)]
struct HookInput {
    session_id: String,
    transcript_path: PathBuf,
}

/// one assistant answer of a transcript
struct Answer {
    time: i64,
    model: String,
    usage: Usage,
}

/// complete new lines of `path` since the last call, as answers
#[cfg(test)]
fn new_answers(path: &Path) -> Result<Vec<Answer>, Box<dyn Error>> {
    Ok(read_lines(path)?.answers)
}

/// what new transcript lines tell: the answers, the time of every line
/// (user and assistant) and the agent's turns (a prompt of the user until
/// the last assistant line before the next prompt)
struct Lines {
    answers: Vec<Answer>,
    times: Vec<i64>,
    turns: Vec<(i64, i64)>,
}

/// a prompt typed by the user (tool results come as user lines too)
fn is_prompt(v: &serde_json::Value) -> bool {
    let c = &v["message"]["content"];
    v["type"] == "user"
        && (c.is_string() || c.as_array().is_some_and(|a| !a.iter().any(|x| x["type"] == "tool_result")))
}

/// complete new lines of `path` since the last call: the answers and the
/// time of every line (user and assistant), for the session log
fn read_lines(path: &Path) -> Result<Lines, Box<dyn Error>> {
    let offsets_file = state_dir()?.join("claude-offsets.json");
    let mut offsets: BTreeMap<String, u64> = read_json(&offsets_file);
    let key = path.to_string_lossy().to_string();
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let mut offset = offsets.get(&key).copied().unwrap_or(0);
    if offset > len {
        offset = 0; // rewritten transcript
    }
    f.seek(SeekFrom::Start(offset))?;
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    let complete = text.rfind('\n').map_or(0, |i| i + 1);
    let mut seen = HashSet::new();
    let mut answers = vec![];
    let mut times = vec![];
    let mut turns = vec![];
    let mut turn: Option<(i64, i64)> = None;
    for line in text[..complete].lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if matches!(v["type"].as_str(), Some("user" | "assistant")) {
            if let Some(t) = v["timestamp"].as_str().and_then(|t| util::parse_timestamp(t).ok()) {
                times.push(t);
                if is_prompt(&v) {
                    turns.extend(turn.take().filter(|(a, b)| b > a));
                    turn = Some((t, t));
                } else if v["type"] == "assistant" {
                    if let Some(tr) = turn.as_mut() {
                        tr.1 = t;
                    }
                }
            }
        }
        let m = &v["message"];
        if v["type"] != "assistant" || m["usage"].is_null() {
            continue;
        }
        // one answer may span several lines with the same id and usage
        if let Some(id) = m["id"].as_str() {
            if !seen.insert(id.to_string()) {
                continue;
            }
        }
        let n = |k: &str| m["usage"][k].as_u64().unwrap_or(0);
        let Some(time) = v["timestamp"].as_str().and_then(|t| util::parse_timestamp(t).ok()) else { continue };
        answers.push(Answer {
            time,
            model: m["model"].as_str().unwrap_or("unknown").to_string(),
            usage: Usage {
                input: n("input_tokens"),
                output: n("output_tokens"),
                cache_read: n("cache_read_input_tokens"),
                cache_write: n("cache_creation_input_tokens"),
            },
        });
    }
    turns.extend(turn.filter(|(a, b)| b > a));
    offsets.insert(key, offset + complete as u64);
    write_json(&offsets_file, &offsets)?;
    Ok(Lines { answers, times, turns })
}

/// the session's activity since the last hook as log lines: stretches of
/// the transcript's times (a pause up to log::GAP after the last stretch
/// continues it), split by the timers of the session's items
fn session_lines(store: &Store, session: &str, times: &[i64], turns: &[(i64, i64)], who: &str) -> Result<Vec<log::Line>, Box<dyn Error>> {
    let ends_file = state_dir()?.join("claude-session-ends.json");
    let mut ends: BTreeMap<String, i64> = read_json(&ends_file);
    let stretches = log::stretches(times, ends.get(session).copied());
    let timers: Vec<(String, i64, i64)> = store
        .get()
        .values()
        .filter(|i| i.sessions().iter().any(|s| s == session))
        .flat_map(|i| i.intervals().into_iter().map(|(s, e)| (i.id().to_owned(), s, e)))
        .collect();
    let mut lines = vec![];
    for (start, end) in &stretches {
        for (s, e, item) in log::split(*start, *end, &timers) {
            let mut l = log::Line::new(s, e, who, "chat");
            l.agent = Some("claude-code".into());
            l.session = Some(session.to_owned());
            l.item = item;
            lines.push(l);
        }
    }
    // the agent's own time: its turns, split by the timers as well
    for (start, end) in turns {
        for (s, e, item) in log::split(*start, *end, &timers) {
            let mut l = log::Line::new(s, e, "claude-code", "agent");
            l.session = Some(session.to_owned());
            l.item = item;
            lines.push(l);
        }
    }
    if let Some((_, end)) = stretches.last() {
        ends.insert(session.to_owned(), *end);
        write_json(&ends_file, &ends)?;
    }
    Ok(lines)
}

/// the Stop hook: attribute the session's new answers and log its
/// activity (with `log_as`: the user); returns the log files written
pub fn claude_hook(store: &mut Store, log_as: Option<&str>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut stdin = String::new();
    std::io::stdin().read_to_string(&mut stdin)?;
    let input: HookInput = serde_json::from_str(&stdin)?;
    let Lines { answers, times, turns } = read_lines(&input.transcript_path)?;
    let files = match log_as {
        Some(who) => log::append(store.root(), &session_lines(store, &input.session_id, &times, &turns, who)?)?,
        None => vec![],
    };
    let file = pending_file(store.root())?;
    let mut pending: Pending = read_json(&file);
    for a in answers {
        let ids: Vec<String> = store
            .get()
            .values()
            .filter(|i| i.sessions().contains(&input.session_id))
            .filter(|i| i.intervals().iter().any(|(s, e)| *s <= a.time && a.time <= *e))
            .map(|i| i.id().to_owned())
            .collect();
        for (n, id) in ids.iter().enumerate() {
            let part = a.usage.split(ids.len() as u64, n == 0);
            *pending.entry(id.clone()).or_default().entry(a.model.clone()).or_default() += part;
        }
    }
    write_json(&file, &pending)?;
    fold(store)?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn transcript_lines_are_read_once_and_answers_deduplicated() {
        let _sandbox = crate::test_support::Sandbox::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.jsonl");
        let line = |id: &str, out: u64| {
            format!(
                "{{\"type\":\"assistant\",\"timestamp\":\"2026-11-13T15:36:40.547Z\",\"message\":{{\"id\":\"{}\",\"model\":\"m\",\"usage\":{{\"input_tokens\":1,\"output_tokens\":{},\"cache_read_input_tokens\":3,\"cache_creation_input_tokens\":4}}}}}}\n",
                id, out
            )
        };
        std::fs::write(&path, format!("{}{}{{\"type\":\"user\"}}\n", line("a", 10), line("a", 10))).unwrap();
        let first = new_answers(&path).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].usage, Usage { input: 1, output: 10, cache_read: 3, cache_write: 4 });
        assert!(new_answers(&path).unwrap().is_empty(), "nothing new");
        let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        std::io::Write::write_all(&mut f, line("b", 5).as_bytes()).unwrap();
        std::io::Write::write_all(&mut f, b"{\"type\":\"assist").unwrap(); // incomplete
        assert_eq!(new_answers(&path).unwrap().len(), 1);
    }
}
