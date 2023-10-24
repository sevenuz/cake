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
    /// if the config is changed, it will be written to config_file
    /// default initialization to false
    #[serde(skip)]
    dirty: bool,
    /// file name searching for when no input file is given (e.g. cake.md)
    save_file_name: String,
    /// full file path to the default save file including file name and suffix (json, md)
    default_file_path: String,
    /// searches in config dir for skin file (hjson)
    skin_file_name: String,
}

const DEFAULT_SKIN_GRUVBOX: &str = r###"# This Hjson file is the default gruvbox skin.
# Checkout following resources for more information:
# https://github.com/Canop/termimad/blob/main/examples/skin-file/skin.hjson
# https://docs.rs/termimad/latest/termimad/struct.MadSkin.html

bold: "#fb0 bold"
italic: dim italic
strikeout: crossedout red
bullet: ○ bold
paragraph: gray(20)
code_block: gray(2) gray(15) left
headers: [
    green bold underlined left
    red underlined left
    yellow left
]
quote: > yellow
horizontal-rule: "~ #00cafe"
table: "#540 left"
scrollbar: "red yellow""###;

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        if let Ok(serialized) = std::fs::read_to_string(util::config_file()?) {
            Ok(serde_json::from_str::<Config>(&serialized)?)
        } else {
            // default settings
            Ok(Config {
                dirty: true,
                save_file_name: "cake.json".to_string(),
                default_file_path: util::default_save_file("cake.json")?,
                skin_file_name: "gruvbox.hjson".to_string(),
            })
        }
    }

    pub fn write_json_if_dirty(&self) -> Result<(), Box<dyn Error>> {
        if self.dirty {
            let serialized = serde_json::to_string_pretty(&self)?;
            std::fs::write(util::config_file()?, serialized)?;
        }
        Ok(())
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
        let path = util::config_path()?.join(&self.skin_file_name);
        let hjson = fs::read_to_string(path.clone()).unwrap_or_else(|_| {
            // write default theme
            let _ = std::fs::write(path, DEFAULT_SKIN_GRUVBOX);
            DEFAULT_SKIN_GRUVBOX.to_string()
        });
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
