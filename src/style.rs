//! one place for how output looks: unicode or ascii characters, color on
//! or off. Set once in `run`, used by every command.

use std::io::IsTerminal;
use std::sync::OnceLock;

pub struct Glyphs {
    pub note: &'static str,
    pub inbox: &'static str,
    pub todo: &'static str,
    pub review: &'static str,
    pub doing: &'static str,
    pub running: &'static str,
    pub done: &'static str,
    pub dropped: &'static str,
    /// tree: item with siblings below, last item, line through, empty
    pub branch: &'static str,
    pub last: &'static str,
    pub pipe: &'static str,
    pub space: &'static str,
    /// between parts of a line
    pub sep: &'static str,
    /// cut text
    pub more: &'static str,
    /// markdown list and rule in `show`
    pub bullet: char,
    pub rule: char,
    /// selected option, done item, free text entry
    pub select: &'static str,
    pub check: &'static str,
    pub ellipsis: &'static str,
    /// progress bar
    pub bar_full: &'static str,
    pub bar_empty: &'static str,
    /// scrollbar thumb
    pub scroll_thumb: &'static str,
    /// saving spinner frames
    pub spinner: &'static [&'static str],
    /// card and header borders
    pub border: ratatui::symbols::border::Set<'static>,
}

const UNICODE: Glyphs = Glyphs {
    note: "·",
    inbox: "◇",
    todo: "○",
    review: "◈",
    doing: "◐",
    running: "▶",
    done: "✓",
    dropped: "✗",
    branch: "├─ ",
    last: "└─ ",
    pipe: "│  ",
    space: "   ",
    sep: " · ",
    more: "…",
    bullet: '•',
    rule: '─',
    select: "›",
    check: "✓",
    ellipsis: "…",
    bar_full: "━",
    bar_empty: "─",
    scroll_thumb: "█",
    spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    border: ratatui::symbols::border::ROUNDED,
};

const ASCII: Glyphs = Glyphs {
    note: "-",
    inbox: "?",
    todo: "o",
    review: "!",
    doing: "*",
    running: ">",
    done: "x",
    dropped: "~",
    branch: "|- ",
    last: "\\- ",
    pipe: "|  ",
    space: "   ",
    sep: ", ",
    more: "...",
    bullet: '-',
    rule: '-',
    select: ">",
    check: "x",
    ellipsis: "...",
    bar_full: "#",
    bar_empty: ".",
    scroll_thumb: "#",
    spinner: &["|", "/", "-", "\\"],
    border: ratatui::symbols::border::Set {
        top_left: "+",
        top_right: "+",
        bottom_left: "+",
        bottom_right: "+",
        vertical_left: "|",
        vertical_right: "|",
        horizontal_top: "-",
        horizontal_bottom: "-",
    },
};

static IS_ASCII: OnceLock<bool> = OnceLock::new();

/// `setting` from the config: auto, unicode or ascii; CAKE_STYLE wins;
/// auto is ascii for TERM=dumb or a locale without UTF-8
pub fn init(setting: &str) {
    let setting = std::env::var("CAKE_STYLE").ok().filter(|s| !s.is_empty()).unwrap_or(setting.to_string());
    let ascii = match setting.as_str() {
        "ascii" => true,
        "unicode" => false,
        _ => {
            let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
                .iter()
                .find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
                .unwrap_or_default()
                .to_lowercase();
            std::env::var("TERM").is_ok_and(|t| t == "dumb") || !(locale.contains("utf-8") || locale.contains("utf8"))
        }
    };
    let _ = IS_ASCII.set(ascii);
}

/// true with the ascii style
pub fn ascii() -> bool {
    *IS_ASCII.get().unwrap_or(&false)
}

pub fn g() -> &'static Glyphs {
    if *IS_ASCII.get().unwrap_or(&false) {
        &ASCII
    } else {
        &UNICODE
    }
}

/// color only on a terminal and without NO_COLOR (no-color.org)
pub fn color() -> bool {
    std::env::var_os("NO_COLOR").map_or(true, |v| v.is_empty()) && std::io::stdout().is_terminal()
}
