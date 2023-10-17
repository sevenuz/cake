use termimad::crossterm::style::Color::*;
use termimad::*;

// markdown style
// should be configurable
// good defaults please :D
pub fn build() -> MadSkin {
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
