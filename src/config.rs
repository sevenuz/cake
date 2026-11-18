use crate::util;
use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

/// ~/.config/cake/config.hjson; every field has a default, a missing file
/// is created from TEMPLATE (with comments and sources), and TEMPLATE is
/// kept as config.default.hjson next to it for reference
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// your name in assignee/owner, else git user.name, else $USER
    pub user: Option<String>,
    pub editor: String,
    pub disable_git: bool,
    pub git_branch: String,
    pub git_remote_name: String,
    pub agent_env: Vec<String>,
    pub session_env: Vec<String>,
    pub status: Vec<String>,
    pub aliases: BTreeMap<String, String>,
    pub style: String,
    pub scrollview_threshold: i8,
    pub form: Vec<String>,
    pub form_width: u16,
    pub tree_depth: usize,
    pub sort_who: String,
    pub keys: BTreeMap<String, String>,
    pub score: crate::store::ScoreWeights,
    pub stats: crate::stats::StatsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            user: None,
            editor: std::env::var("EDITOR").unwrap_or("vim".to_string()),
            disable_git: false,
            git_branch: default_git_branch(),
            git_remote_name: "origin".to_string(),
            agent_env: default_agent_env(),
            session_env: default_session_env(),
            status: default_status(),
            aliases: BTreeMap::new(),
            style: default_style(),
            scrollview_threshold: 30,
            form: crate::form::MODULES.iter().map(|m| m.to_string()).collect(),
            form_width: 100,
            tree_depth: 2,
            sort_who: "ai".to_string(),
            keys: BTreeMap::new(),
            score: Default::default(),
            stats: Default::default(),
        }
    }
}

/// the defaults with explanations; must parse to Config::default()
pub const TEMPLATE: &str = r##"// cake configuration (hjson: comments, quotes optional).
// Remove a line to get its default back. config.default.hjson next to this
// file always holds the current defaults with these comments.
{
  // your name for assignee and owner; default: git user.name, else $USER
  // user: jul
  // editor for cake add / cake edit without -m
  editor: vim
  // no git commits and syncs at all
  disable_git: false
  // branch of a project store (a worktree in .cake) and the remote it syncs with
  git_branch: cake/main
  git_remote_name: origin
  // env vars naming the agent running cake (value cut at the first _) and
  // holding its session id; first set one wins
  agent_env: ["AI_AGENT"]
  session_env: ["CAKE_SESSION", "CLAUDE_CODE_SESSION_ID"]
  // cake status: mine, others, ready, today, week (you and agents), streak
  // (days in a row with time or a done item), review, tag:<name>
  status: ["mine", "others", "ready", "today", "week", "streak", "review"]
  // own commands, may override built-ins for you (not for agents):
  // check: "ls --status review", ls: "ls --mine".
  // A line starting with ! is a shell command, like in git, e.g. for
  // Claude Code (run them yourself, `cake triage`, or from cron):
  //   triage: plans only, the AI may use nothing but cake:
  //     "!claude -p 'Triage this cake store with the cake skill: for tasks assigned to ai without subtasks, split them into subtasks or ask questions; apply answers and comments; garden the tree (file top-level items under goals, ask about overlaps); do not implement anything; end with a short summary.' --max-turns 40 --allowedTools 'Bash(cake *)'"
  //   work: the full agent run; it edits code, so it needs Read, Edit,
  //   Write and the build tools of the project, e.g.
  //     "!claude -p 'Run the agent run of the cake skill for this store until nothing is ready or the reviews are full, then a short summary.' --max-turns 80 --allowedTools 'Bash(cake *)' 'Bash(cargo *)' 'Bash(git *)' Read Edit Write"
  //   The prompt comes right after -p: --allowedTools takes several values
  //   and would swallow it. --max-turns is the emergency stop; the normal
  //   brake is max_review in the store config.
  aliases: {}
  // characters: auto (ascii without a UTF-8 locale), unicode, ascii;
  // env CAKE_STYLE wins. Colors: theme.hjson, off with NO_COLOR.
  style: auto
  // lines from which cake show opens a pager, -1 never
  scrollview_threshold: 30
  // modules of `cake` in order: news (what agents did), inbox (inbox and
  // tasks without assignee), answer (questions for you), review (your
  // finished work), mine (your tasks),
  // dump (type ideas into the inbox); `cake form review` runs one
  form: ["news", "inbox", "answer", "review", "mine"]
  // widest column of the form, 0 the whole terminal
  form_width: 100
  // parents the form shows above an item, deeper ones as "… ›"
  tree_depth: 2
  // who a task from the inbox module gets: "ai", "me", a name, "-" nobody
  sort_who: "ai"
  // form keys, vim notation, several per action; defaults:
  //   down "j <down>", up "k <up>", top "gg <home>", bottom "G <end>",
  //   half_down "<c-d> <pagedown>", half_up "<c-u> <pageup>",
  //   prev_module "h <left>", next_module "l <right>",
  //   open "o <space>", back "<esc>", confirm "<cr>", insert "i",
  //   task "t", note "N", drop "dd x", complete "c", timer "s", who "w",
  //   prio "P", undo "u", next "n J", previous "p K",
  //   add "a", add_child "A", focus "<tab>", editor "e", feedback "F", command ":",
  //   help "?", quit "q",
  //   escape (leaving text input) "<esc> jk"; cards choose with 1-9
  keys: {}

  // order of ls, ready and the form: higher first
  score: {
    // hard deadline: deadline_today points on the day, +/- per day, capped
    deadline_today: 100
    deadline_per_day: 10
    deadline_max: 200
    // soft due date, same scheme
    due_today: 50
    due_per_day: 5
    due_max: 100
    // per open item waiting on it (blocked_by), transitively
    per_waiting: 10
    // priority 1, 2, 3
    prio: [30, 20, 10]
    // one point per day of age, up to
    age_max: 20
  }

  // what stats sum beyond time and done items: named formulas over the
  // items' fields, columns, overlay sections; timers merged by key. An
  // example for agent tokens and their footprint: skill/hooks/stats.example.hjson
  stats: {
    merge_by: ["sessions", "by"]
    sums: []
    show: []
    sections: []
  }
}
"##;

fn default_git_branch() -> String {
    "cake/main".to_string()
}

fn default_agent_env() -> Vec<String> {
    vec!["AI_AGENT".to_string()]
}

fn default_session_env() -> Vec<String> {
    vec!["CAKE_SESSION".to_string(), "CLAUDE_CODE_SESSION_ID".to_string()]
}

fn default_style() -> String {
    "auto".to_string()
}

fn default_status() -> Vec<String> {
    ["mine", "others", "ready", "today", "week", "streak", "review"].iter().map(|s| s.to_string()).collect()
}

fn first_env(vars: &[String]) -> Option<String> {
    vars.iter().find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
}

/// committed per store in .cake/config.hjson, shared by everyone using the store
#[derive(Deserialize)]
#[serde(default)]
pub struct StoreConfig {
    pub aliases: BTreeMap<String, String>,
    /// humans who may be assigned; empty: anyone
    pub people: Vec<String>,
    /// agents who may be assigned besides `ai`; empty: anyone
    pub agents: Vec<String>,
    /// while this many items wait in review, agents get no new work; -1 off
    pub max_review: i64,
    /// free header fields: types and derived formulas (see fields.rs)
    pub fields: crate::fields::Defs,
}

impl Default for StoreConfig {
    fn default() -> Self {
        StoreConfig { aliases: BTreeMap::new(), people: vec![], agents: vec![], max_review: 8, fields: Default::default() }
    }
}

impl Config {
    /// default aliases, then the user's, then the store's
    pub fn aliases(&self, store: &std::path::Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        let mut aliases: BTreeMap<String, String> = [
            ("inbox", "ls --status inbox"),
            ("check", "ls --status review"),
            ("ready", "ls --ready"),
            ("done", "set done"),
            ("drop", "set dropped"),
            ("do", "set task"),
            ("task", "set task"),
            ("todo", "set task"),
            ("reopen", "set task"),
            ("note", "set note"),
            ("review", "set review"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        aliases.extend(self.aliases.clone());
        aliases.extend(Self::store(store)?.aliases);
        Ok(aliases)
    }

    /// the store's own config, defaults without a file
    pub fn store(store: &std::path::Path) -> Result<StoreConfig, Box<dyn Error>> {
        let file = store.join("config.hjson");
        if !file.is_file() {
            return Ok(StoreConfig::default());
        }
        Ok(deser_hjson::from_str(&fs::read_to_string(&file)?).map_err(|e| format!("{}: {}", file.display(), e))?)
    }

    /// `me` is you, `ai` any agent, `-` nobody; other names must be in the
    /// store's people or agents when those are set
    pub fn who(&self, store: &std::path::Path, input: &str) -> Result<Option<String>, Box<dyn Error>> {
        let me = self.user(store);
        match input {
            "-" => return Ok(None),
            "me" => return Ok(Some(me)),
            "ai" => return Ok(Some("ai".into())),
            _ => (),
        }
        let sc = Self::store(store)?;
        let known: Vec<&String> = sc.people.iter().chain(&sc.agents).collect();
        if !known.is_empty() && input != me && !known.iter().any(|k| *k == input) {
            let names: Vec<&str> = known.iter().map(|k| k.as_str()).collect();
            return Err(crate::error::invalid(format!(
                "unknown assignee '{}': use me, ai, {} (store config people/agents)",
                input,
                names.join(", ")
            )));
        }
        Ok(Some(input.to_string()))
    }

    /// AI agent running cake, from `agent_env`
    pub fn agent(&self) -> Option<String> {
        first_env(&self.agent_env).map(|a| a.split('_').next().unwrap().to_string())
    }

    /// agent session changing items, from `session_env`
    pub fn session(&self) -> Option<String> {
        first_env(&self.session_env)
    }

    /// the human using cake, also when an agent runs it: CAKE_AUTHOR, else git
    /// user.name, else $USER
    pub fn user(&self, store: &std::path::Path) -> String {
        first_env(&["CAKE_AUTHOR".to_string()])
            .or_else(|| self.user.clone())
            .or_else(|| crate::git::user_name(store))
            .or_else(|| std::env::var("USER").ok())
            .unwrap_or("unknown".to_string())
    }

    /// creator of new items: CAKE_AUTHOR, else the agent, else git user.name, else $USER
    pub fn author(&self, store: &std::path::Path) -> String {
        first_env(&["CAKE_AUTHOR".to_string()])
            .or_else(|| self.agent())
            .or_else(|| self.user.clone())
            .or_else(|| crate::git::user_name(store))
            .or_else(|| std::env::var("USER").ok())
            .unwrap_or("unknown".to_string())
    }

    /// reads the config, creates it from TEMPLATE if missing and refreshes
    /// config.default.hjson
    pub fn new() -> Result<Config, Box<dyn Error>> {
        let file = util::config_file()?;
        let reference = util::config_path()?.join("config.default.hjson");
        if fs::read_to_string(&reference).ok().as_deref() != Some(TEMPLATE) {
            let _ = fs::write(&reference, TEMPLATE);
        }
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(_) => {
                let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
                let text = TEMPLATE.replacen("editor: vim", &format!("editor: {}", editor), 1);
                let _ = fs::write(&file, &text);
                text
            }
        };
        Ok(deser_hjson::from_str::<Config>(&text).map_err(|e| format!("{}: {}", file.display(), e))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Sandbox;

    #[test]
    fn template_parses_to_the_defaults() {
        let parsed: Config = deser_hjson::from_str(TEMPLATE).unwrap();
        let json = |c: &Config| {
            let mut v = serde_json::to_value(c).unwrap();
            v["editor"] = "x".into();
            v
        };
        assert_eq!(json(&parsed), json(&Config::default()));
    }

    #[test]
    #[serial_test::serial]
    fn missing_config_is_written_from_the_template_with_comments() {
        let _sandbox = Sandbox::new();
        Config::new().unwrap();
        let text = fs::read_to_string(util::config_file().unwrap()).unwrap();
        assert!(text.contains("// what stats sum") && text.contains("stats.example.hjson"));
        let reference = util::config_path().unwrap().join("config.default.hjson");
        assert_eq!(fs::read_to_string(reference).unwrap(), TEMPLATE);
    }

    #[test]
    #[serial_test::serial]
    fn new_returns_sane_defaults_when_no_config_file_exists() {
        let _sandbox = Sandbox::new();
        let config = Config::new().unwrap();
        assert_eq!(config.git_branch, "cake/main");
        assert_eq!(config.git_remote_name, "origin");
        assert!(!config.disable_git);
    }

    #[test]
    #[serial_test::serial]
    fn agent_session_and_author_from_configured_env() {
        let _sandbox = Sandbox::new();
        let mut config = Config::new().unwrap();
        config.agent_env = vec!["TEST_AGENT".into()];
        config.session_env = vec!["TEST_SESSION".into()];
        let vars = ["CAKE_AUTHOR", "TEST_AGENT", "TEST_SESSION"];
        let saved: Vec<_> = vars.iter().map(|v| std::env::var(v).ok()).collect();
        vars.iter().for_each(|v| std::env::remove_var(v));

        assert_eq!(config.agent(), None);
        assert_eq!(config.session(), None);
        std::env::set_var("TEST_AGENT", "some-tool_1_agent");
        std::env::set_var("TEST_SESSION", "s1");
        assert_eq!(config.agent().as_deref(), Some("some-tool"));
        assert_eq!(config.session().as_deref(), Some("s1"));
        assert_eq!(config.author(std::path::Path::new("/")), "some-tool");
        std::env::set_var("CAKE_AUTHOR", "jul");
        assert_eq!(config.author(std::path::Path::new("/")), "jul");

        for (v, s) in vars.iter().zip(saved) {
            match s {
                Some(s) => std::env::set_var(v, s),
                None => std::env::remove_var(v),
            }
        }
    }
}
