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
    Add,
    AddChild,
    Focus,
    Filter,
    Editor,
    Feedback,
    Command,
    Help,
    Quit,
    /// q without a binding: says how to quit
    QuitHint,
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
    ("prev_module", Act::PrevModule, "h <left>", "previous module"),
    ("next_module", Act::NextModule, "l <right>", "next module"),
    ("open", Act::Open, "o <space>", "show or hide parent, siblings and children"),
    ("back", Act::Back, "<esc>", "close, cancel"),
    ("confirm", Act::Confirm, "<cr>", "confirm: save the option, open the box"),
    ("insert", Act::Insert, "i", "own words, comment, change request"),
    ("task", Act::Task, "t", "make it a task"),
    ("note", Act::Note, "N", "keep it as a note"),
    ("drop", Act::Drop, "dd x", "drop"),
    ("complete", Act::Complete, "c", "done"),
    ("timer", Act::Timer, "s", "start or stop the timer"),
    ("who", Act::Who, "w", "who: you, ai, agents, people"),
    ("prio", Act::Prio, "P", "priority 1, 2, 3, none"),
    ("undo", Act::Undo, "u", "undo the row's decision"),
    ("next", Act::Next, "n J", "next item that waits for a decision"),
    ("previous", Act::Previous, "p K", "previous item that waits for a decision"),
    ("add", Act::Add, "a", "new item into the inbox"),
    ("add_child", Act::AddChild, "A", "new item under the hovered row"),
    ("focus", Act::Focus, "<tab>", "switch between the tree and the details"),
    ("filter", Act::Filter, "f", "all: open work, + notes, + closed"),
    ("editor", Act::Editor, "e", "open the item in the editor"),
    ("feedback", Act::Feedback, "F", "feedback on this screen"),
    ("command", Act::Command, ":", "command line: due, deadline, who, prio, move, all, w, q"),
    ("help", Act::Help, "?", "all keys"),
    ("quit", Act::Quit, "", "quit; no key by default, use :q (<c-c> always quits)"),
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
    Delete,
    Left,
    Right,
    WordLeft,
    WordRight,
    Home,
    End,
    /// <c-w>: delete the word before the cursor
    DeleteWord,
}

/// a line being typed with a cursor in it (chars before the cursor: `at`)
#[derive(Default, Debug, PartialEq)]
pub struct Buf {
    chars: Vec<char>,
    pub at: usize,
}

impl Buf {
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// start of the word left of `at`: spaces first, then the word
    fn word_left(&self) -> usize {
        let mut n = self.at;
        while n > 0 && self.chars[n - 1] == ' ' {
            n -= 1;
        }
        while n > 0 && self.chars[n - 1] != ' ' {
            n -= 1;
        }
        n
    }

    /// end of the word right of `at`: spaces first, then the word
    fn word_right(&self) -> usize {
        let mut n = self.at;
        while n < self.chars.len() && self.chars[n] == ' ' {
            n += 1;
        }
        while n < self.chars.len() && self.chars[n] != ' ' {
            n += 1;
        }
        n
    }

    /// applies an editing key; Submit and Escape are the caller's
    pub fn apply(&mut self, key: &Text) {
        match key {
            Text::Char('\0') | Text::Submit | Text::Escape => (),
            Text::Char(c) => {
                self.chars.insert(self.at, *c);
                self.at += 1;
            }
            Text::Backspace if self.at > 0 => {
                self.at -= 1;
                self.chars.remove(self.at);
            }
            Text::Backspace => (),
            Text::Delete if self.at < self.chars.len() => {
                self.chars.remove(self.at);
            }
            Text::Delete => (),
            Text::Left => self.at = self.at.saturating_sub(1),
            Text::Right => self.at = (self.at + 1).min(self.chars.len()),
            Text::WordLeft => self.at = self.word_left(),
            Text::WordRight => self.at = self.word_right(),
            Text::Home => self.at = 0,
            Text::End => self.at = self.chars.len(),
            Text::DeleteWord => {
                let from = self.word_left();
                self.chars.drain(from..self.at);
                self.at = from;
            }
        }
    }
}

pub struct Keymap {
    binds: Vec<(Vec<K>, Act)>,
    escapes: Vec<Vec<K>>,
    /// keys read but not used yet (after a sequence did not match)
    pending: VecDeque<K>,
    /// when keys were pressed, for the session log
    pub active: Vec<i64>,
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
        Ok(Keymap { binds, escapes, pending: VecDeque::new(), active: vec![crate::util::timestamp()] })
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
                    self.active.push(crate::util::timestamp());
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
                    if first == K::char('q') {
                        return Ok(Act::QuitHint);
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
            KeyCode::Delete => Text::Delete,
            KeyCode::Left if first.ctrl => Text::WordLeft,
            KeyCode::Right if first.ctrl => Text::WordRight,
            KeyCode::Left => Text::Left,
            KeyCode::Right => Text::Right,
            KeyCode::Home => Text::Home,
            KeyCode::End => Text::End,
            KeyCode::Char('a') if first.ctrl => Text::Home,
            KeyCode::Char('e') if first.ctrl => Text::End,
            KeyCode::Char('w') if first.ctrl => Text::DeleteWord,
            KeyCode::Char(_) if first.ctrl => Text::Char('\0'),
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
                        let tmp = Keymap { binds: vec![(s.clone(), *act)], escapes: vec![], pending: VecDeque::new(), active: vec![] };
                        tmp.keys_of(*act)
                    })
                    .collect();
                let all = if all.is_empty() && *act == Act::Quit { vec![":q <c-c>".to_string()] } else { all };
                (all.join(" "), what.to_string())
            })
            .collect();
        out.insert(8, ("1-9".into(), "cards: choose option n".into()));
        // the legend of the marks in the tree, first
        let g = crate::style::g();
        let legend = [
            (format!("{} {} {} {}", g.inbox, g.todo, g.doing, g.review), "marks: inbox, task, doing, review".to_string()),
            (format!("{} {} {}", g.note, g.done, g.dropped), format!("marks: note, done, dropped; {} before a title: changed this run", g.check)),
            (format!("? +N {} {}", g.ellipsis, g.select), "a question for you; N children hidden (o shows); parents above cut".to_string()),
            ("ALL".to_string(), "your own items in their own color (theme `mine`), agents' plain".to_string()),
            (String::new(), String::new()),
        ];
        out.splice(0..0, legend);
        out.push(("<esc> jk".into(), "text: leave without saving".into()));
        out.push(("<left> <right>".into(), "text: move; with <c-…> a word; <home> <end> <c-a> <c-e>".into()));
        out.push(("<del> <c-w>".into(), "text: delete the char under the cursor, the word before it".into()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buf_edits_at_the_cursor_and_jumps_words() {
        let mut b = Buf::default();
        for c in "fix teh typo".chars() {
            b.apply(&Text::Char(c));
        }
        b.apply(&Text::WordLeft);
        b.apply(&Text::WordLeft);
        assert_eq!(b.at, 4, "start of teh");
        b.apply(&Text::WordRight);
        assert_eq!(b.at, 7, "end of teh");
        b.apply(&Text::Backspace);
        b.apply(&Text::Backspace);
        b.apply(&Text::Char('h'));
        b.apply(&Text::Char('e'));
        assert_eq!(b.text(), "fix the typo");
        b.apply(&Text::Home);
        b.apply(&Text::Delete);
        assert_eq!(b.text(), "ix the typo");
        b.apply(&Text::End);
        b.apply(&Text::DeleteWord);
        assert_eq!(b.text(), "ix the ");
        b.apply(&Text::Left);
        b.apply(&Text::Right);
        b.apply(&Text::Right);
        assert_eq!(b.at, 7, "right stops at the end");
    }

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
