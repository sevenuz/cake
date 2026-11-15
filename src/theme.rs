//! colors of all output, from ~/.config/cake/theme.hjson (written with the
//! defaults when missing). Characters come from `style`, this is color only.

use crate::style;
use ratatui::crossterm::style::{Color as Term, Stylize};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::error::Error;
use std::str::FromStr;
use std::sync::OnceLock;

pub const FILE: &str = "theme.hjson";

const DEFAULT: &str = r##"// cake colors: names (red, lightgreen, gray, darkgray, ...),
// #rrggbb or 0-255. Remove a line to get the default back.
{
  // markdown in cake show and the form
  heading: "#b8bb26"
  bold: "#fabd2f"
  code: "#d3869b"
  quote: "#fabd2f"
  // lists and status
  dim: "gray"
  running: "#b8bb26"
  doing: "#fabd2f"
  // form
  question: "#fabd2f"
  hint: "#83a598"
  // your own items in the ALL tree (agents' items stay plain)
  mine: "#fe8019"
  // #tags in the tree and the details
  tag: "#8ec07c"
  ok: "#b8bb26"
  error: "#fb4934"
}
"##;

#[derive(Deserialize)]
#[serde(default)]
struct Raw {
    heading: String,
    bold: String,
    code: String,
    quote: String,
    dim: String,
    running: String,
    doing: String,
    question: String,
    hint: String,
    mine: String,
    tag: String,
    ok: String,
    error: String,
}

/// same values as DEFAULT
impl Default for Raw {
    fn default() -> Self {
        let s = |v: &str| v.to_string();
        Raw {
            heading: s("#b8bb26"),
            bold: s("#fabd2f"),
            code: s("#d3869b"),
            quote: s("#fabd2f"),
            dim: s("gray"),
            running: s("#b8bb26"),
            doing: s("#fabd2f"),
            question: s("#fabd2f"),
            hint: s("#83a598"),
            mine: s("#fe8019"),
            tag: s("#8ec07c"),
            ok: s("#b8bb26"),
            error: s("#fb4934"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub heading: Color,
    pub bold: Color,
    pub code: Color,
    pub quote: Color,
    pub dim: Color,
    pub running: Color,
    pub doing: Color,
    pub question: Color,
    pub hint: Color,
    pub mine: Color,
    pub tag: Color,
    pub ok: Color,
    pub error: Color,
}

static THEME: OnceLock<Theme> = OnceLock::new();

fn parse(name: &str, value: &str) -> Result<Color, Box<dyn Error>> {
    Color::from_str(value).map_err(|_| format!("{}: '{}' for {} is not a color", FILE, value, name).into())
}

/// reads the theme file once, writes the default if there is none
pub fn init() -> Result<(), Box<dyn Error>> {
    let path = crate::util::config_path()?.join(FILE);
    let raw: Raw = match std::fs::read_to_string(&path) {
        Ok(text) => deser_hjson::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e))?,
        Err(_) => {
            let _ = std::fs::write(&path, DEFAULT);
            Raw::default()
        }
    };
    let theme = Theme {
        heading: parse("heading", &raw.heading)?,
        bold: parse("bold", &raw.bold)?,
        code: parse("code", &raw.code)?,
        quote: parse("quote", &raw.quote)?,
        dim: parse("dim", &raw.dim)?,
        running: parse("running", &raw.running)?,
        doing: parse("doing", &raw.doing)?,
        question: parse("question", &raw.question)?,
        hint: parse("hint", &raw.hint)?,
        mine: parse("mine", &raw.mine)?,
        tag: parse("tag", &raw.tag)?,
        ok: parse("ok", &raw.ok)?,
        error: parse("error", &raw.error)?,
    };
    let _ = THEME.set(theme);
    Ok(())
}

pub fn t() -> Theme {
    *THEME.get_or_init(|| {
        let raw = Raw::default();
        Theme {
            heading: parse("", &raw.heading).unwrap(),
            bold: parse("", &raw.bold).unwrap(),
            code: parse("", &raw.code).unwrap(),
            quote: parse("", &raw.quote).unwrap(),
            dim: parse("", &raw.dim).unwrap(),
            running: parse("", &raw.running).unwrap(),
            doing: parse("", &raw.doing).unwrap(),
            question: parse("", &raw.question).unwrap(),
            hint: parse("", &raw.hint).unwrap(),
            mine: parse("", &raw.mine).unwrap(),
            tag: parse("", &raw.tag).unwrap(),
            ok: parse("", &raw.ok).unwrap(),
            error: parse("", &raw.error).unwrap(),
        }
    })
}

fn term(c: Color) -> Term {
    match c {
        Color::Reset => Term::Reset,
        Color::Black => Term::Black,
        Color::Red => Term::DarkRed,
        Color::Green => Term::DarkGreen,
        Color::Yellow => Term::DarkYellow,
        Color::Blue => Term::DarkBlue,
        Color::Magenta => Term::DarkMagenta,
        Color::Cyan => Term::DarkCyan,
        Color::Gray => Term::Grey,
        Color::DarkGray => Term::DarkGrey,
        Color::LightRed => Term::Red,
        Color::LightGreen => Term::Green,
        Color::LightYellow => Term::Yellow,
        Color::LightBlue => Term::Blue,
        Color::LightMagenta => Term::Magenta,
        Color::LightCyan => Term::Cyan,
        Color::White => Term::White,
        Color::Rgb(r, g, b) => Term::Rgb { r, g, b },
        Color::Indexed(i) => Term::AnsiValue(i),
    }
}

/// `text` in a ratatui style as ANSI for printed (non TUI) output;
/// plain without color (NO_COLOR, pipes)
pub fn paint(text: &str, s: Style) -> String {
    if !style::color() || text.is_empty() {
        return text.to_string();
    }
    let mut out = text.stylize();
    if let Some(fg) = s.fg {
        out = out.with(term(fg));
    }
    if let Some(bg) = s.bg {
        out = out.on(term(bg));
    }
    if s.add_modifier.contains(Modifier::BOLD) {
        out = out.bold();
    }
    if s.add_modifier.contains(Modifier::ITALIC) {
        out = out.italic();
    }
    if s.add_modifier.contains(Modifier::UNDERLINED) {
        out = out.underlined();
    }
    out.to_string()
}

/// shorthand: `text` in color `c`
pub fn fg(text: &str, c: Color) -> String {
    paint(text, Style::default().fg(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_parses_and_bad_colors_are_named() {
        let raw: Raw = deser_hjson::from_str(DEFAULT).unwrap();
        assert_eq!(raw.error, Raw::default().error, "DEFAULT and Default agree");
        assert_eq!(parse("heading", &raw.heading).unwrap(), Color::Rgb(0xb8, 0xbb, 0x26));
        assert_eq!(parse("dim", "darkgray").unwrap(), Color::DarkGray);
        assert_eq!(parse("dim", "42").unwrap(), Color::Indexed(42));
        let err = parse("dim", "greyish").unwrap_err().to_string();
        assert!(err.contains("'greyish' for dim"), "{}", err);
    }
}
