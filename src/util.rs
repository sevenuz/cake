use platform_dirs::AppDirs;
use std::{
    env::temp_dir,
    error::Error,
    fs::{read_dir, write, File},
    io::Read,
    path::PathBuf,
    process::Command,
};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use nanoid::nanoid;

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
    ];

    nanoid!(3, &alphabet)
}

pub fn timestamp() -> Duration {
    let t = SystemTime::now();
    t.duration_since(UNIX_EPOCH).unwrap()
}

pub fn split_comma(s: String) -> Vec<String> {
    // return empty vector if input is ""
    if s == "" {
        return vec![];
    }
    // TODO improvement!!!
    let ca: Vec<&str> = s.split(",").collect();
    let mut vec: Vec<String> = Vec::new();
    ca.into_iter().for_each(|ll| {
        vec.push(ll.to_string());
    });
    return vec;
}

pub fn remove_illegal_characters(mut s: String) -> String {
    s = s.replace("<", "");
    s = s.replace(">", "");
    s.replace("!", "")
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
