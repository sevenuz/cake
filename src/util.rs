use crate::error;
use chrono::{DateTime, Local, TimeZone};
use directories::{BaseDirs, ProjectDirs};
use nanoid::nanoid;
use std::{
    env::temp_dir,
    error::Error,
    fs::{self, read_dir, write, File},
    io::Read,
    path::PathBuf,
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
    fn test_str_to_vec() {
        assert_eq!(str_to_vec("1, 2, 3"), vec!["1", "2", "3"]);
    }

    #[test]
    fn test_vec_to_str() {
        assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
        assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
    }

    // NOTE: assumes the host is running in the CET/CEST (+0100/+0200) timezone,
    // same assumption the original version of this test made.
    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(1678197184), "Tue Mar  7 14:53:04 2023 +0100");
    }

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(
            parse_timestamp("Tue Mar  7 14:53:04 2023 +0100").unwrap(),
            1678197184
        );
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
    fn test_extract_metadata_strips_prefix_and_suffix() {
        assert_eq!(extract_metadata("| id | abc|", "| id | ").unwrap(), "abc");
    }

    #[test]
    fn test_extract_metadata_errs_on_missing_prefix() {
        assert!(extract_metadata("nope|", "| id | ").is_err());
    }

    #[test]
    fn test_extract_metadata_errs_on_missing_suffix() {
        assert!(extract_metadata("| id | abc", "| id | ").is_err());
    }

    #[test]
    fn test_generate_id_uses_only_alphabet_characters_and_length_three() {
        let id = generate_id();
        assert_eq!(id.len(), 3);
        assert!(id
            .chars()
            .all(|c| "asdfghjklertuinv".contains(c)));
    }

    #[test]
    #[serial]
    fn find_save_file_locates_file_in_an_ancestor_directory() {
        let sandbox = Sandbox::new();
        std::fs::create_dir_all("a/b/c").unwrap();
        std::fs::write("a/cake.md", "").unwrap();

        let mut start = sandbox.path().join("a/b/c");
        let found = find_save_file(&mut start, "cake.md").unwrap();

        assert_eq!(found, sandbox.path().join("a/cake.md").to_str().unwrap());
    }

    #[test]
    #[serial]
    fn find_save_file_falls_back_to_default_when_not_found() {
        let sandbox = Sandbox::new();
        let mut start = sandbox.path().to_path_buf();

        let found = find_save_file(&mut start, "cake.md").unwrap();

        assert_eq!(found, default_save_file("cake.md").unwrap());
    }
}

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'e', 'r', 't', 'u', 'i', 'n', 'v',
    ];

    nanoid!(3, &alphabet)
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

// DO NOT CHANGE IT, it will break parse_timestamp
const DATE_FORMAT: &str = "%c %z";
pub fn format_timestamp(timestamp: i64) -> String {
    Local
        .timestamp_opt(timestamp, 0)
        .unwrap()
        .format(DATE_FORMAT)
        .to_string()
}

/// from the formatted string of fn ft, this parses the unix timestamp
pub fn parse_timestamp(s: &str) -> Result<i64, error::ParseError> {
    let dt_timestamp = DateTime::parse_from_str(s, DATE_FORMAT)
        .ok()
        .ok_or(error::ParseError {
            message: "Invalid timestamp".to_string(),
        })?;
    return Ok(dt_timestamp.timestamp());
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

pub fn default_save_file(save_file_name: &str) -> Result<String, Box<dyn Error>> {
    if let Some(base_dirs) = BaseDirs::new() {
        let path = base_dirs.data_dir(); // ~/.local/.. on linux
        fs::create_dir_all(path)?; // Do we need this?
        return Ok(path
            .join(save_file_name)
            .into_os_string()
            .into_string()
            .unwrap());
    } else {
        return Err("Could not find data directory".into());
    }
}

/// find next cake save file in current or upper dirs, fallback is data_dir
pub fn find_save_file(path: &mut PathBuf, save_file_name: &str) -> Result<String, Box<dyn Error>> {
    if path.is_dir() {
        for entry in read_dir(path.as_path())? {
            let path = entry?.path();
            let name = path.file_name().ok_or("No filename")?;

            if name == save_file_name {
                return Ok(path.into_os_string().into_string().unwrap());
            }
        }
    }

    if path.pop() {
        return find_save_file(path, save_file_name);
    } else {
        return default_save_file(save_file_name);
    }
}

/// checks if at least one element of vec1 is contained in vec2
pub fn contains_element<T>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool
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
pub fn is_subset<T>(vec1: &Vec<T>, vec2: &Vec<T>) -> bool
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

/// splits string seperated by comma
pub fn str_to_vec(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(", ").map(|v| v.to_string()).collect()
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

/// removes prefix and suffix from raw metadata line
pub fn extract_metadata(s: &str, prefix: &str) -> Result<String, error::ParseError> {
    Ok(s.strip_prefix(prefix)
        .ok_or(error::ParseError {
            message: "Invalid metadata".to_string(),
        })?
        .strip_suffix("|")
        .ok_or(error::ParseError {
            message: "Invalid metadata".to_string(),
        })?
        .to_string())
}
