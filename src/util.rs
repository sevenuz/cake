use nanoid::nanoid;
use platform_dirs::AppDirs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    env::temp_dir,
    error::Error,
    fs::{read_dir, write, File},
    io::Read,
    path::PathBuf,
    process::Command,
};

#[cfg(test)]
mod tests {
    use super::*;

    //#[should_panic]
    #[test]
    fn test_parse_time() {
        assert_eq!(
            parse_time("3s").unwrap().unwrap(),
            timestamp().as_secs() - 3
        );
        assert_eq!(
            parse_time("1h").unwrap().unwrap(),
            timestamp().as_secs() - 60 * 60
        );
        assert_eq!(
            parse_time("30m").unwrap().unwrap(),
            timestamp().as_secs() - 30 * 60
        );
        assert_eq!(
            parse_time("30m1s4w").unwrap().unwrap(),
            timestamp().as_secs() - (30 * 60 + 1 + 4 * 7 * 24 * 60 * 60)
        );
        assert_eq!(
            parse_time("1y1d").unwrap().unwrap(),
            timestamp().as_secs() - (365 * 24 * 60 * 60 + 24 * 60 * 60)
        );
        assert!(parse_time("100sc1hwac3h1sinn").is_err());
        assert!(parse_time("1T").is_err());
        assert!(parse_time("0.5Y").is_err());
    }
}

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
    ];

    nanoid!(3, &alphabet)
}

// parse e.g. 1y1w1d1h1m1s and subtracts it from now
pub fn parse_time(t: &str) -> Result<Option<u64>, Box<dyn Error>> {
    if t.is_empty() {
        return Ok(None);
    }

    const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    const UNIT: [char; 6] = ['y', 'w', 'd', 'h', 'm', 's'];
    let mut values: [u64; 6] = [0, 0, 0, 0, 0, 0];

    let mut last_pos = 0;
    for (i, c) in t.chars().enumerate() {
        if UNIT.contains(&c) {
            match t[last_pos..i].parse::<u64>() {
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
    let now = timestamp().as_secs();
    Ok(Some(now - t))
}

pub fn timestamp() -> Duration {
    let t = SystemTime::now();
    t.duration_since(UNIX_EPOCH).unwrap()
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
    s
}

const NAME: &str = env!("CARGO_PKG_NAME");
const SAVE_FILE: &str = "cake.json";

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

    Command::new(editor)
        .arg(&file_path)
        .status()
        .expect("Something went wrong");

    let mut editable = String::new();
    File::open(file_path)
        .expect("Could not open file")
        .read_to_string(&mut editable)?;
    Ok(editable)
}

// find next cake save file in current or upper dirs, fallback is data_dir
pub fn find_save_file(path: &mut PathBuf) -> Result<String, Box<dyn Error>> {
    if path.is_dir() {
        for entry in read_dir(path.as_path())? {
            let path = entry?.path();
            let name = path.file_name().ok_or("No filename")?;

            if name == SAVE_FILE {
                return Ok(path.into_os_string().into_string().unwrap());
            }
        }
    }

    if path.pop() {
        return find_save_file(path);
    } else {
        let app_dirs = AppDirs::new(Some(NAME), false).unwrap();
        return Ok(app_dirs
            .data_dir
            .join(SAVE_FILE)
            .into_os_string()
            .into_string()
            .unwrap());
    }
}
