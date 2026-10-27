use crate::error;
use chrono::{DateTime, Local, SecondsFormat, TimeZone};
use directories::{BaseDirs, ProjectDirs};
use nanoid::nanoid;
use std::{
    env::temp_dir,
    error::Error,
    fs::{self, write, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Sandbox;
    use serial_test::serial;

    //#[should_panic]
    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("3s").unwrap().unwrap(), timestamp() - 3);
        assert_eq!(parse_time("1h").unwrap().unwrap(), timestamp() - 60 * 60);
        assert_eq!(parse_time("30m").unwrap().unwrap(), timestamp() - 30 * 60);
        assert_eq!(
            parse_time("30m1s4w").unwrap().unwrap(),
            timestamp() - (30 * 60 + 1 + 4 * 7 * 24 * 60 * 60)
        );
        assert_eq!(
            parse_time("1y1d").unwrap().unwrap(),
            timestamp() - (365 * 24 * 60 * 60 + 24 * 60 * 60)
        );
        assert!(parse_time("100sc1hwac3h1sinn").is_err());
        assert!(parse_time("1T").is_err());
        assert!(parse_time("0.5Y").is_err());
    }


    #[test]
    fn test_vec_to_str() {
        assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
        assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
    }

    #[test]
    fn test_timestamp_round_trip() {
        assert_eq!(parse_timestamp(&format_timestamp(1678197184)).unwrap(), 1678197184);
        assert_eq!(parse_timestamp("2023-03-07T14:53:04+01:00").unwrap(), 1678197184);
    }

    #[test]
    fn test_timestamp_to_hms() {
        assert_eq!(timestamp_to_hms(1000), "16m40s");
    }

    #[test]
    fn test_space_pads_evenly_on_both_sides() {
        assert_eq!(space("ab", 6), "  ab  ");
    }

    #[test]
    fn test_space_extra_padding_goes_on_the_right_when_odd() {
        assert_eq!(space("ab", 5), " ab  ");
    }

    #[test]
    #[should_panic]
    fn test_space_panics_when_string_is_longer_than_spacer_len() {
        // BUG: `space` computes `spacer_len - s.len()` with no bounds check,
        // so an id longer than spacer_len panics on unsigned-subtraction
        // overflow instead of e.g. returning the string unpadded.
        space("toolong", 3);
    }

    #[test]
    fn test_split_comma_cleanup() {
        assert_eq!(
            split_comma_cleanup(" a , b ,,c ".to_string()),
            vec!["a", "b", "c"]
        );
        assert_eq!(split_comma_cleanup("".to_string()), Vec::<String>::new());
    }

    #[test]
    fn test_split_comma_tags_excludes_tilde_prefixed_entries() {
        assert_eq!(
            split_comma_tags("work, ~blocked, urgent".to_string()),
            vec!["work", "urgent"]
        );
    }

    #[test]
    fn test_split_comma_exclude_tags_keeps_only_tilde_prefixed_entries() {
        assert_eq!(
            split_comma_exclude_tags("work, ~blocked, urgent".to_string()),
            vec!["blocked"]
        );
    }

    #[test]
    fn test_remove_illegal_characters_strips_leading_tilde_and_pipes() {
        assert_eq!(remove_illegal_characters("~a|b|c".to_string()), "abc");
    }

    #[test]
    fn test_contains_element() {
        assert!(contains_element(&vec!["a", "b"], &vec!["b", "c"]));
        assert!(!contains_element(&vec!["a"], &vec!["b", "c"]));
    }

    #[test]
    fn test_is_subset() {
        assert!(is_subset(&vec!["a", "b"], &vec!["a", "b", "c"]));
        assert!(!is_subset(&vec!["a", "d"], &vec!["a", "b", "c"]));
    }




    #[test]
    fn test_generate_id_uses_only_alphabet_characters_and_length_six() {
        let id = generate_id();
        assert_eq!(id.len(), 6);
        assert!(id.chars().all(|c| "0123456789abcdefghjkmnpqrstvwxyz".contains(c)));
    }

    #[test]
    #[serial]
    fn find_store_locates_store_in_an_ancestor_directory() {
        let sandbox = Sandbox::new();
        std::fs::create_dir_all("a/b/c").unwrap();
        std::fs::create_dir_all("a/.cake/items").unwrap();

        let found = find_store(&sandbox.path().join("a/b/c")).unwrap();

        assert_eq!(found, sandbox.path().join("a/.cake"));
    }

    #[test]
    #[serial]
    fn agent_and_author_from_env() {
        let _sandbox = Sandbox::new();
        let vars = ["CAKE_AUTHOR", "AI_AGENT", "CLAUDECODE"];
        let saved: Vec<_> = vars.iter().map(|v| std::env::var(v).ok()).collect();
        vars.iter().for_each(|v| std::env::remove_var(v));

        assert_eq!(agent(), None);
        std::env::set_var("CLAUDECODE", "1");
        assert_eq!(agent().as_deref(), Some("claude-code"));
        std::env::set_var("AI_AGENT", "some-tool_1_agent");
        assert_eq!(agent().as_deref(), Some("some-tool"));
        assert_eq!(author(Path::new("/")), "some-tool");
        std::env::set_var("CAKE_AUTHOR", "jul");
        assert_eq!(author(Path::new("/")), "jul");

        for (v, s) in vars.iter().zip(saved) {
            match s {
                Some(s) => std::env::set_var(v, s),
                None => std::env::remove_var(v),
            }
        }
    }

    #[test]
    #[serial]
    fn find_store_falls_back_to_global_store() {
        let sandbox = Sandbox::new();
        assert_eq!(find_store(sandbox.path()).unwrap(), global_store().unwrap());
    }
}

pub fn generate_id() -> String {
    let alphabet: Vec<char> = "0123456789abcdefghjkmnpqrstvwxyz".chars().collect();
    nanoid!(6, &alphabet)
}

/// parse e.g. 1y1w1d1h1m1s and subtracts it from now
pub fn parse_time(t: &str) -> Result<Option<i64>, Box<dyn Error>> {
    if t.is_empty() {
        return Ok(None);
    }

    const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    const UNIT: [char; 6] = ['y', 'w', 'd', 'h', 'm', 's'];
    let mut values: [i64; 6] = [0, 0, 0, 0, 0, 0];

    let mut last_pos = 0;
    for (i, c) in t.chars().enumerate() {
        if UNIT.contains(&c) {
            match t[last_pos..i].parse::<i64>() {
                Err(err) => return Err(Box::new(err)),
                Ok(n) => {
                    values[UNIT.iter().position(|e| *e == c).unwrap()] = n;
                }
            }
            last_pos = i + 1;
        } else if !NUMBERS.contains(&c) {
            return Err("Wrong time format".into());
        }
    }
    let t = 365 * 24 * 60 * 60 * values[0] // years
        + 7 * 24 * 60 * 60 * values[1] // week
        + 24 * 60 * 60 * values[2] // days
        + 60 * 60 * values[3] // hours
        + 60 * values[4] // minutes
        + values[5]; // seconds
    let now = timestamp();
    Ok(Some(now - t))
}

/// show timestamp in hours, minutes, seconds
pub fn timestamp_to_hms(timestamp: i64) -> String {
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

/// RFC 3339 in local time, second precision
pub fn format_timestamp(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .unwrap()
        .to_rfc3339_opts(SecondsFormat::Secs, false)
}

pub fn parse_timestamp(s: &str) -> Result<i64, error::ParseError> {
    Ok(DateTime::parse_from_rfc3339(s.trim())
        .map_err(|_| error::ParseError {
            message: format!("Invalid timestamp '{}'", s),
        })?
        .timestamp())
}

pub fn timestamp() -> i64 {
    Local::now().timestamp()
}

pub fn split_comma_tags(s: String) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(",")
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter(|s| s.chars().next().unwrap() != '~')
        .map(|s| remove_illegal_characters(s.to_string()))
        .collect()
}

pub fn split_comma_exclude_tags(s: String) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(",")
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter(|s| s.chars().next().unwrap() == '~')
        .map(|s| remove_illegal_characters(s.to_string()))
        .collect()
}

// return empty vector if input is empty
// removes illegal characters
pub fn split_comma_cleanup(s: String) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(",")
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| remove_illegal_characters(s.to_string()))
        .collect()
}

fn remove_illegal_characters(mut s: String) -> String {
    // remove ~ when on start of str
    // because it is used as exclude sign
    if s.chars().next().unwrap() == '~' {
        s = s.replacen("~", "", 1);
    }
    // not allowed because pipe is used in metadata header of markdown representation
    s.replace("|", "")
}

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CONFIG_FILE: &str = "config.hjson";

pub fn input_from_external_editor(
    editor: &str,
    text: Option<&String>,
) -> Result<String, Box<dyn Error>> {
    let mut file_path = temp_dir();
    file_path.push("editable.md");
    File::create(&file_path).expect("Could not create file.");

    if let Some(t) = text {
        write(&file_path, t).expect("Could not write to file.");
    }

    Command::new(editor).arg(&file_path).status()?;

    let mut editable = String::new();
    File::open(file_path)?.read_to_string(&mut editable)?;
    Ok(editable)
}

pub fn config_file() -> Result<PathBuf, Box<dyn Error>> {
    return Ok(config_path()?.join(CONFIG_FILE));
}

pub fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    // TODO random qualifier and organization string :D
    // https://github.com/dirs-dev/directories-rs#projectdirs
    if let Some(proj_dirs) = ProjectDirs::from("net", "sevenuz", PKG_NAME) {
        let path = proj_dirs.config_dir();
        fs::create_dir_all(path)?;
        return Ok(path.to_path_buf());
    } else {
        return Err("Could not find config directory".into());
    }
}

/// AI agent running cake, from AI_AGENT (e.g. claude-code_2-1-308_agent ->
/// claude-code) or CLAUDECODE=1
pub fn agent() -> Option<String> {
    if let Some(a) = std::env::var("AI_AGENT").ok().filter(|a| !a.is_empty()) {
        return Some(a.split('_').next().unwrap().to_string());
    }
    (std::env::var("CLAUDECODE").as_deref() == Ok("1")).then(|| "claude-code".to_string())
}

/// creator of new items: CAKE_AUTHOR, else the agent, else git user.name, else $USER
pub fn author(store: &Path) -> String {
    std::env::var("CAKE_AUTHOR")
        .ok()
        .filter(|a| !a.is_empty())
        .or_else(agent)
        .or_else(|| crate::git::user_name(store))
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or("unknown".to_string())
}

pub const STORE_DIR: &str = ".cake";

/// the store's lock file, None while the store dir does not exist
pub fn lock_file(root: &Path) -> Result<Option<fd_lock::RwLock<File>>, Box<dyn Error>> {
    if !root.is_dir() {
        return Ok(None);
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(".lock"))?;
    Ok(Some(fd_lock::RwLock::new(file)))
}

/// held, not read: dropping it releases the lock
#[allow(dead_code)]
pub enum LockGuard<'a> {
    Shared(fd_lock::RwLockReadGuard<'a, File>),
    Exclusive(fd_lock::RwLockWriteGuard<'a, File>),
}

/// blocks until the lock is free, says so once on stderr if it has to wait;
/// the OS releases it when the process ends, also on a crash
pub fn lock(lock: &mut fd_lock::RwLock<File>, exclusive: bool) -> Result<LockGuard<'_>, Box<dyn Error>> {
    let busy = |r: std::io::Result<()>| matches!(r, Err(e) if e.kind() == std::io::ErrorKind::WouldBlock);
    let waiting = if exclusive {
        busy(lock.try_write().map(drop))
    } else {
        busy(lock.try_read().map(drop))
    };
    if waiting {
        eprintln!("waiting for lock (another cake command is running)");
    }
    Ok(if exclusive {
        LockGuard::Exclusive(lock.write()?)
    } else {
        LockGuard::Shared(lock.read()?)
    })
}

pub const STORE_GITIGNORE: &str = "index.sqlite\n.lock\n.*.md.tmp\n.obsidian/workspace*.json\n";

pub fn global_store() -> Result<PathBuf, Box<dyn Error>> {
    let base_dirs = BaseDirs::new().ok_or("Could not find data directory")?;
    Ok(base_dirs.data_dir().join(PKG_NAME))
}

/// find next store with an items dir in current or upper dirs, fallback is the global store
pub fn find_store(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    for dir in path.ancestors() {
        if dir.join(STORE_DIR).join("items").is_dir() {
            return Ok(dir.join(STORE_DIR));
        }
    }
    global_store()
}

/// checks if at least one element of vec1 is contained in vec2
pub fn contains_element<T>(vec1: &[T], vec2: &[T]) -> bool
where
    T: PartialEq,
{
    for i in vec1 {
        if vec2.contains(i) {
            return true;
        }
    }
    false
}

/// checks if all elements of vec1 are contained in vec2
pub fn is_subset<T>(vec1: &[T], vec2: &[T]) -> bool
where
    T: PartialEq,
{
    for i in vec1 {
        if !vec2.contains(i) {
            return false;
        }
    }
    true
}

pub fn space(s: &str, spacer_len: usize) -> String {
    let prefix = (spacer_len - s.len()) / 2;
    let appendix = if (spacer_len - s.len()) % 2 == 0 {
        prefix
    } else {
        prefix + 1
    };
    let mut result = (0..prefix).map(|_| " ").collect::<String>();
    result += &s;
    result += &(0..appendix).map(|_| " ").collect::<String>();
    result
}

/// joins vector with comma
pub fn vec_to_str<T>(v: &Vec<T>) -> String
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
