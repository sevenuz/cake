use crate::util;
use crate::Error;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs;
use termimad::MadSkin;

// color scheme of madskin in separat hjson file -> gruvbox default theme
// skin_path in config file
// default saving location -> md or json
// aliases for run cmd or hook cmds?
// default selectors
// default modifiers for one or multiple hits
// hire_recursive_elements
// default editor or environment var?
// .cake folder in ~ with default md/json file?
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// file name searching for when no input file is given (e.g. cake.md)
    save_file_name: String,
    /// full file path to the default save file including file name and suffix (json, md)
    default_file_path: String,
    /// searches in config dir for skin file (hjson)
    skin_file_name: String,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        if let Ok(serialized) = std::fs::read_to_string(util::config_file()?) {
            Ok(serde_json::from_str::<Config>(&serialized)?)
        } else {
            // default settings
            Ok(Config {
                save_file_name: "cake.json".to_string(),
                default_file_path: util::default_save_file("cake.json")?,
                skin_file_name: "gruvbox.hjson".to_string(),
            })
        }
    }

    pub fn find_save_file(&self) -> Result<String, Box<dyn Error>> {
        util::find_save_file(&mut current_dir()?, &self.save_file_name)
    }

    pub fn get_default_file_path(&self) -> String {
        self.default_file_path.to_string()
    }

    // markdown style
    // should be configurable
    // good defaults please :D
    pub fn build_skin(&self) -> Result<MadSkin, Box<dyn Error>> {
        // https://github.com/Canop/termimad/blob/main/examples/skin-file/main.rs
        // read the skin file in a string
        let hjson = fs::read_to_string(util::config_path()?.join(&self.skin_file_name))?;
        // deserialize the Hjson into a skin
        let skin: MadSkin = deser_hjson::from_str(&hjson)?;

        // let mut skin = MadSkin::default();
        // skin.set_headers_fg(rgb(255, 187, 0));
        // skin.headers[0].set_fg(rgb(155, 187, 0));
        // skin.headers[1].set_fg(rgb(155, 87, 0));
        // skin.headers[0].align = Alignment::Left;
        // skin.headers[1].align = Alignment::Left;
        // skin.headers[2].align = Alignment::Left;
        // skin.bold.set_fg(Yellow);
        // skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
        // skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
        // skin.quote_mark.set_fg(Yellow);
        return Ok(skin);
    }
}
