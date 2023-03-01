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

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
    ];

    nanoid!(3, &alphabet)
}

pub fn parse_time(t: &str) -> Option<u64> {
    if t.is_empty() {
        return None;
    }
    let t = SystemTime::now();
    Some(t.duration_since(UNIX_EPOCH).unwrap().as_secs())
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
