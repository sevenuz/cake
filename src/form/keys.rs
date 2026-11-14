//! the form's keys: vim-like defaults, configurable per action in config
//! `keys` with vim notation ("dd x", "<c-d>", "<esc> jk"). Normal mode
//! returns actions; text input returns characters, with escape sequences
//! like `jk` recognized within ESCAPE_MS.

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::time::Duration;

/// how long a started sequence (dd, gg, jk) waits for its next key
const SEQUENCE_MS: u64 = 1000;
/// jk-style escapes in text must be typed this fast, like vim's timeoutlen
const ESCAPE_MS: u64 = 300;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Down,
    Up,
    Top,
    Bottom,
    HalfDown,
    HalfUp,
    Open,
    Back,
    PrevModule,
    NextModule,
    Confirm,
    Insert,
    Task,
    Note,
    Drop,
    Complete,
    Timer,
    Who,
    Prio,
    Undo,
    Next,
    Previous,
    Subtree,
    FullTree,
    Feedback,
    Command,
    Help,
    Quit,
    /// a digit without a binding: choose option n
    Choose(u8),
}

/// (config name, action, default keys, what it does)
pub const ACTIONS: &[(&str, Act, &str, &str)] = &[
    ("down", Act::Down, "j <down>", "down"),
    ("up", Act::Up, "k <up>", "up"),
    ("top", Act::Top, "gg <home>", "first"),
    ("bottom", Act::Bottom, "G <end>", "last"),
    ("half_down", Act::HalfDown, "<c-d> <pagedown>", "scroll the text, half a page down"),
    ("half_up", Act::HalfUp, "<c-u> <pageup>", "scroll the text, half a page up"),
    ("prev_module", Act::PrevModule, "h <left>", "previous module (saves the table)"),
    ("next_module", Act::NextModule, "l <right>", "next module (saves the table)"),
    ("open", Act::Open, "o <space>", "open the text"),
    ("back", Act::Back, "<esc>", "close, cancel"),
    ("confirm", Act::Confirm, "<cr>", "confirm (cards), save (tables)"),
    ("insert", Act::Insert, "i", "own words, comment, change request"),
    ("task", Act::Task, "t", "make it a task"),
    ("note", Act::Note, "n", "keep it as a note"),
    ("drop", Act::Drop, "dd x", "drop"),
    ("complete", Act::Complete, "c", "done"),
    ("timer", Act::Timer, "s", "start or stop the timer"),
    ("who", Act::Who, "w", "who: you, ai, agents, people"),
    ("prio", Act::Prio, "p", "priority 1, 2, 3, none"),
    ("undo", Act::Undo, "u", "undo the row's decision"),
    ("next", Act::Next, "J", "next item without deciding"),
    ("previous", Act::Previous, "K", "previous item"),
    ("subtree", Act::Subtree, "A", "sort: this row's decision for the rows below"),
    ("full_tree", Act::FullTree, "O", "the whole tree around this row, read only"),
    ("feedback", Act::Feedback, "F", "feedback on this screen"),
    ("command", Act::Command, ":", "command line: due, deadline, who, prio, move, w, q"),
    ("help", Act::Help, "?", "all keys"),
    ("quit", Act::Quit, "q", "quit"),
];

/// escape in text input, the config name `escape`
const ESCAPE_DEFAULT: &str = "<esc> jk";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct K {
    pub code: KeyCode,
    pub ctrl: bool,
}

impl K {
    fn from(e: KeyEvent) -> K {
        K { code: e.code, ctrl: e.modifiers.contains(KeyModifiers::CONTROL) }
    }

    fn char(c: char) -> K {
        K { code: KeyCode::Char(c), ctrl: false }
    }
}

/// one binding token: "dd", "<c-d>", "gg", "<esc>"
pub fn parse_seq(token: &str) -> Result<Vec<K>, String> {
    let mut out = vec![];
    let mut chars = token.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '<' || chars.peek().is_none() {
            out.push(K::char(c));
            continue;
        }
        let name: String = chars.by_ref().take_while(|c| *c != '>').collect();
        let lower = name.to_lowercase();
        let (ctrl, base) = match lower.strip_prefix("c-") {
            Some(b) => (true, b.to_string()),
            None => (false, lower.clone()),
        };
        let code = match base.as_str() {
            "esc" => KeyCode::Esc,
            "cr" | "enter" | "return" => KeyCode::Enter,
            "bs" | "backspace" => KeyCode::Backspace,
            "tab" => KeyCode::Tab,
            "space" => KeyCode::Char(' '),
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "lt" => KeyCode::Char('<'),
            b if b.chars().count() == 1 => KeyCode::Char(name.chars().last().unwrap()),
            _ => return Err(format!("unknown key <{}>", name)),
        };
        out.push(K { code, ctrl });
    }
    Ok(out)
}

fn parse_spec(spec: &str) -> Result<Vec<Vec<K>>, String> {
    spec.split_whitespace().map(parse_seq).collect()
}

/// what text input gets
pub enum Text {
    Char(char),
    Submit,
    Escape,
    Backspace,
}

pub struct Keymap {
    binds: Vec<(Vec<K>, Act)>,
    escapes: Vec<Vec<K>>,
    /// keys read but not used yet (after a sequence did not match)
    pending: VecDeque<K>,
}

impl Keymap {
    /// defaults, then config `keys` (action name -> keys) over them
    pub fn new(config: &BTreeMap<String, String>) -> Result<Keymap, Box<dyn Error>> {
        if let Some(bad) = config.keys().find(|k| *k != "escape" && !ACTIONS.iter().any(|(n, ..)| n == k)) {
            let names: Vec<&str> = ACTIONS.iter().map(|(n, ..)| *n).collect();
            return Err(format!("config keys: unknown action '{}', use {}, escape", bad, names.join(", ")).into());
        }
        let mut binds = vec![];
        for (name, act, default, _) in ACTIONS {
            let spec = config.get(*name).map(String::as_str).unwrap_or(default);
            for seq in parse_spec(spec).map_err(|e| format!("config keys.{}: {}", name, e))? {
                binds.push((seq, *act));
            }
        }
        let escapes = parse_spec(config.get("escape").map(String::as_str).unwrap_or(ESCAPE_DEFAULT))
            .map_err(|e| format!("config keys.escape: {}", e))?;
        Ok(Keymap { binds, escapes, pending: VecDeque::new() })
    }

    /// the keys of `act` as written in vim notation, for help and legends
    pub fn keys_of(&self, act: Act) -> String {
        let show = |k: &K| {
            let base = match k.code {
                KeyCode::Char(' ') => "<space>".to_string(),
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Enter => "<cr>".into(),
                KeyCode::Esc => "<esc>".into(),
                KeyCode::Up => "<up>".into(),
                KeyCode::Down => "<down>".into(),
                KeyCode::Left => "<left>".into(),
                KeyCode::Right => "<right>".into(),
                other => format!("{:?}", other).to_lowercase(),
            };
            if k.ctrl { format!("<c-{}>", base.trim_matches(|c| c == '<' || c == '>')) } else { base }
        };
        let list: Vec<String> = self
            .binds
            .iter()
            .filter(|(_, a)| *a == act)
            .map(|(seq, _)| seq.iter().map(show).collect::<String>())
            .collect();
        list.first().cloned().unwrap_or_default()
    }

    fn raw(&mut self, wait: Option<u64>) -> Result<Option<K>, Box<dyn Error>> {
        if let Some(k) = self.pending.pop_front() {
            return Ok(Some(k));
        }
        loop {
            if let Some(ms) = wait {
                if !event::poll(Duration::from_millis(ms))? {
                    return Ok(None);
                }
            }
            if let Event::Key(e) = event::read()? {
                if e.kind == KeyEventKind::Press {
                    return Ok(Some(K::from(e)));
                }
            }
        }
    }

    /// the next action in normal mode; ctrl-c always quits; keys without a
    /// binding are skipped, digits choose
    pub fn act(&mut self) -> Result<Act, Box<dyn Error>> {
        loop {
            let first = self.raw(None)?.unwrap();
            if first.ctrl && first.code == KeyCode::Char('c') {
                return Ok(Act::Quit);
            }
            let mut seq = vec![first];
            loop {
                if let Some(act) = self.binds.iter().find(|(s, _)| *s == seq).map(|(_, a)| *a) {
                    // a longer binding with this prefix (g vs gg) waits briefly
                    let longer = self.binds.iter().any(|(s, _)| s.len() > seq.len() && s.starts_with(&seq));
                    if !longer {
                        return Ok(act);
                    }
                    match self.raw(Some(SEQUENCE_MS))? {
                        None => return Ok(act),
                        Some(k) => {
                            seq.push(k);
                            if !self.binds.iter().any(|(s, _)| s.starts_with(&seq)) {
                                self.pending.push_back(seq.pop().unwrap());
                                return Ok(act);
                            }
                        }
                    }
                    continue;
                }
                if self.binds.iter().any(|(s, _)| s.len() > seq.len() && s.starts_with(&seq)) {
                    match self.raw(Some(SEQUENCE_MS))? {
                        Some(k) => {
                            seq.push(k);
                            continue;
                        }
                        None => break,
                    }
                }
                if seq.len() == 1 {
                    if let KeyCode::Char(c @ '1'..='9') = first.code {
                        return Ok(Act::Choose(c as u8 - b'0'));
                    }
                }
                break;
            }
        }
    }

    /// the next key of text input; escape sequences like jk must be typed
    /// within ESCAPE_MS, otherwise their letters are text
    pub fn text(&mut self) -> Result<Text, Box<dyn Error>> {
        let first = self.raw(None)?.unwrap();
        if first.ctrl && first.code == KeyCode::Char('c') {
            return Ok(Text::Escape);
        }
        let mut seq = vec![first];
        loop {
            if self.escapes.iter().any(|e| *e == seq) {
                return Ok(Text::Escape);
            }
            if !self.escapes.iter().any(|e| e.len() > seq.len() && e.starts_with(&seq)) {
                break;
            }
            match self.raw(Some(ESCAPE_MS))? {
                Some(k) => seq.push(k),
                None => break,
            }
        }
        // no escape: the first key counts, the rest is read again
        for k in seq.drain(1..).rev() {
            self.pending.push_front(k);
        }
        Ok(match first.code {
            KeyCode::Enter => Text::Submit,
            KeyCode::Esc => Text::Escape,
            KeyCode::Backspace => Text::Backspace,
            KeyCode::Char(c) => Text::Char(c),
            _ => Text::Char('\0'),
        })
    }

    /// all bindings for the help overlay: (keys, what)
    pub fn help(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = ACTIONS
            .iter()
            .map(|(_, act, _, what)| {
                let all: Vec<String> = self
                    .binds
                    .iter()
                    .filter(|(_, a)| a == act)
                    .map(|(s, _)| {
                        let tmp = Keymap { binds: vec![(s.clone(), *act)], escapes: vec![], pending: VecDeque::new() };
                        tmp.keys_of(*act)
                    })
                    .collect();
                (all.join(" "), what.to_string())
            })
            .collect();
        out.insert(8, ("1-9".into(), "cards: choose option n".into()));
        out.push(("<esc> jk".into(), "text: leave without saving".into()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vim_notation_parses() {
        assert_eq!(parse_seq("dd").unwrap(), vec![K::char('d'), K::char('d')]);
        assert_eq!(parse_seq("<c-d>").unwrap(), vec![K { code: KeyCode::Char('d'), ctrl: true }]);
        assert_eq!(parse_seq("<Esc>").unwrap(), vec![K { code: KeyCode::Esc, ctrl: false }]);
        assert_eq!(parse_seq("<").unwrap(), vec![K::char('<')]);
        assert!(parse_seq("<nope>").is_err());
    }

    #[test]
    fn config_overrides_and_checks_names() {
        let mut c = BTreeMap::new();
        c.insert("drop".to_string(), "D".to_string());
        let m = Keymap::new(&c).unwrap();
        assert_eq!(m.keys_of(Act::Drop), "D");
        assert_eq!(m.keys_of(Act::Down), "j");
        c.insert("dorp".to_string(), "x".to_string());
        assert!(Keymap::new(&c).err().unwrap().to_string().contains("unknown action 'dorp'"));
    }
}
