use serde::{Deserialize, Serialize};
use termimad::crossterm::style::Color::*;
use termimad::*;

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
    skin_path: String,
}
// markdown style
// should be configurable
// good defaults please :D
pub fn build_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(rgb(255, 187, 0));
    skin.headers[0].set_fg(rgb(155, 187, 0));
    skin.headers[1].set_fg(rgb(155, 87, 0));
    skin.headers[0].align = Alignment::Left;
    skin.headers[1].align = Alignment::Left;
    skin.headers[2].align = Alignment::Left;
    skin.bold.set_fg(Yellow);
    skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
    skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
    skin.quote_mark.set_fg(Yellow);
    return skin;
}
