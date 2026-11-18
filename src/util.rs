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
    fn test_timestamp_round_trip() {
        assert_eq!(parse_timestamp(&format_timestamp(1678197184)).unwrap(), 1678197184);
        assert_eq!(parse_timestamp("2023-03-07T14:53:04+01:00").unwrap(), 1678197184);
    }

    #[test]
    fn test_timestamp_to_hms() {
        assert_eq!(timestamp_to_hms(1000), "16m40s");
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
        assert!(id.chars().all(|c| "abcdefghjkmnpqrstuvwxyz".contains(c)));
    }

    #[test]
    fn parse_date_understands_short_forms() {
        use chrono::{Datelike, Duration};
        let today = Local::now().date_naive();
        let fmt = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();
        assert_eq!(parse_date("2026-11-30").unwrap(), "2026-11-30");
        assert_eq!(parse_date("today").unwrap(), fmt(today));
        assert_eq!(parse_date("Tomorrow").unwrap(), fmt(today + Duration::days(1)));
        assert_eq!(parse_date("+3d").unwrap(), fmt(today + Duration::days(3)));
        assert_eq!(parse_date("2w").unwrap(), fmt(today + Duration::days(14)));
        assert_eq!(parse_age("3m").unwrap(), 90 * 86400);
        assert_eq!(timestamp_to_hms(3700), "1h1m40s");
        assert_eq!(timestamp_to_hms(9000), "2h30m");
        assert_eq!(parse_age("2w").unwrap(), 14 * 86400);
        assert!(parse_age("3").is_err() && parse_age("xm").is_err() && parse_age("3h").is_err());
        assert_eq!(parse_date("1w2d").unwrap(), fmt(today + Duration::days(9)));
        let fri = parse_date("fri").unwrap();
        let fri = chrono::NaiveDate::parse_from_str(&fri, "%Y-%m-%d").unwrap();
        assert_eq!(fri.weekday(), chrono::Weekday::Fri);
        assert!(fri >= today && fri < today + Duration::days(7));
        assert_eq!(parse_date("friday").unwrap(), fmt(fri));
        assert_eq!(parse_date("30.11.2027").unwrap(), "2027-11-30");
        let next = parse_date("1.1.").unwrap();
        assert!(next > fmt(today) && next.ends_with("-01-01"), "{}", next);
        for bad in ["soon", "fr", "31.2.", "+3x", "frix"] {
            assert!(parse_date(bad).is_err(), "{}", bad);
        }
    }

    #[test]
    fn free_id_skips_taken_ids() {
        let first = free_id(|_| false);
        let other = free_id(|id| id == first);
        assert_eq!(other.len(), 6);
        assert_ne!(first, other);
    }

    #[test]
    fn check_id_refuses_path_characters() {
        assert!(check_id("merge-driver").is_ok());
        assert!(check_id("a/b").is_err());
        assert!(check_id(".hidden").is_err());
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
    fn find_store_falls_back_to_global_store() {
        let sandbox = Sandbox::new();
        assert_eq!(find_store(sandbox.path()).unwrap(), global_store().unwrap());
    }
}

/// a random id that `taken` does not hold yet
pub fn free_id(taken: impl Fn(&str) -> bool) -> String {
    std::iter::repeat_with(generate_id).find(|id| !taken(id)).unwrap()
}

/// ids are file and directory names in the store
pub fn check_id(id: &str) -> Result<(), Box<dyn Error>> {
    if id.trim().is_empty() || id.starts_with('.') || id.contains(['/', '\\']) {
        return Err(error::invalid(format!("'{}' is not a valid id (no '/', '\\', leading '.').", id)));
    }
    if id == crate::store::inner::ARCHIVE {
        return Err(error::invalid(format!("'{}' is reserved for the archive directory.", id)));
    }
    Ok(())
}

/// an age like 3m (months of 30 days), 2w, 10d or 1y, in seconds
pub fn parse_age(s: &str) -> Result<i64, Box<dyn Error>> {
    let s = s.trim();
    let bad = || error::invalid(format!("'{}' is no age, e.g. 3m (months), 2w, 10d, 1y.", s));
    let unit = s.chars().last().ok_or_else(bad)?;
    let n: i64 = s[..s.len() - unit.len_utf8()].parse().map_err(|_| bad())?;
    let day = 24 * 60 * 60;
    Ok(n * match unit {
        'd' => day,
        'w' => 7 * day,
        'm' => 30 * day,
        'y' => 365 * day,
        _ => return Err(bad()),
    })
}

/// a date (end of that day, local) or an RFC 3339 datetime as unix seconds
pub fn date_timestamp(s: &str) -> Option<i64> {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d.and_hms_opt(23, 59, 59)?.and_local_timezone(Local).earliest().map(|t| t.timestamp());
    }
    parse_timestamp(s).ok()
}

pub fn generate_id() -> String {
    // letters only, without i, l and o, which are easy to misread
    let alphabet: Vec<char> = "abcdefghjkmnpqrstuvwxyz".chars().collect();
    nanoid!(6, &alphabet)
}

/// parse e.g. 1y1w1d1h1m1s and subtracts it from now
pub fn parse_time(t: &str) -> Result<Option<i64>, Box<dyn Error>> {
    if t.is_empty() {
        return Ok(None);
    }
    Ok(Some(timestamp() - parse_duration(t)?))
}

/// seconds of e.g. 1y1w1d1h1m1s
pub fn parse_duration(t: &str) -> Result<i64, Box<dyn Error>> {
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
    Ok(t)
}

/// a due date or deadline as YYYY-MM-DD (RFC 3339 times stay): today,
/// tomorrow, mon..sun (the next one, today included), +3d / 2w1d (from
/// now), 30.11. or 30.11.2026 (day.month, without year the next one)
pub fn parse_date(s: &str) -> Result<String, String> {
    use chrono::{Datelike, Duration, NaiveDate};
    let s = s.trim();
    let err = || format!("'{}' is no date: use 2026-11-30, 30.11., today, tomorrow, fri, +3d, 2w", s);
    if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() || parse_timestamp(s).is_ok() {
        return Ok(s.to_string());
    }
    let today = Local::now().date_naive();
    let lower = s.to_lowercase();
    let day = match lower.as_str() {
        "today" => Some(today),
        "tomorrow" => Some(today + Duration::days(1)),
        _ => None,
    };
    let weekdays = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let weekday = weekdays.iter().position(|w| lower.len() >= 3 && w.starts_with(&lower[..3]) && "monday tuesday wednesday thursday friday saturday sunday".contains(lower.as_str()));
    let day = day.or_else(|| {
        weekday.map(|w| {
            let ahead = (w as i64 - today.weekday().num_days_from_monday() as i64).rem_euclid(7);
            today + Duration::days(ahead)
        })
    });
    let day = day.or_else(|| {
        let d = lower.strip_prefix('+').unwrap_or(&lower);
        d.chars().next().filter(|c| c.is_ascii_digit())?;
        let secs = parse_duration(d).ok()?;
        Some((Local::now() + Duration::seconds(secs)).date_naive())
    });
    let day = day.or_else(|| {
        let parts: Vec<&str> = lower.trim_end_matches('.').split('.').collect();
        let (d, m) = (parts.first()?.parse::<u32>().ok()?, parts.get(1)?.parse::<u32>().ok()?);
        match parts.get(2) {
            Some(y) => NaiveDate::from_ymd_opt(y.parse::<i32>().ok()?, m, d),
            None => NaiveDate::from_ymd_opt(today.year(), m, d)
                .filter(|date| *date >= today)
                .or_else(|| NaiveDate::from_ymd_opt(today.year() + 1, m, d)),
        }
    });
    day.map(|d| d.format("%Y-%m-%d").to_string()).ok_or_else(err)
}

/// show timestamp in hours, minutes, seconds
pub fn timestamp_to_hms(timestamp: i64) -> String {
    let hours = timestamp / 60 / 60;
    let minutes = (timestamp - hours * 60 * 60) / 60;
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

/// the nearest .cake with an items dir, else a listed store whose dirs hold
/// `path` (stores.json), else the global store
pub fn find_store(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    for dir in path.ancestors() {
        if dir.join(STORE_DIR).join("items").is_dir() {
            return Ok(dir.join(STORE_DIR));
        }
    }
    if let Some(store) = crate::stores::for_dir(&crate::stores::load()?, path) {
        return Ok(store);
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

