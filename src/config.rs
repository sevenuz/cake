use crate::util;
use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

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
    /// editor for cake add
    pub editor: String,
    /// disable all git actions
    pub disable_git: bool,
    /// branch of the project store worktree
    #[serde(default = "default_git_branch")]
    pub git_branch: String,
    /// remote used by cake init and cake sync --publish
    pub git_remote_name: String,
    /// env vars naming the AI agent running cake, first set one wins,
    /// value cut at the first _ (claude-code_2-1_agent -> claude-code)
    #[serde(default = "default_agent_env")]
    pub agent_env: Vec<String>,
    /// env vars holding the agent's session id, first set one wins
    #[serde(default = "default_session_env")]
    pub session_env: Vec<String>,
    /// cake status segments: mine (your running timers), others (count of
    /// others' running timers), ready, today (time tracked today in the store,
    /// overlaps once), tag:<name> (open items with that tag)
    #[serde(default = "default_status")]
    pub status: Vec<String>,
    /// own commands: name -> command line, e.g. review: "ls -t review"; may
    /// override built-ins (ls: "ls --status todo"); added to the defaults, a
    /// store's .cake/config.hjson overrides them
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
    /// characters: auto (ascii without a UTF-8 locale), unicode or ascii;
    /// env CAKE_STYLE wins. Color follows NO_COLOR and the terminal.
    #[serde(default = "default_style")]
    pub style: String,
    /// When output has more lines than the threshold, then show it in scrollview
    /// -1 to never go into scrollview
    /// 127 as maximum
    pub scrollview_threshold: i8,
}

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
    ["mine", "others", "ready", "today", "tag:review"].iter().map(|s| s.to_string()).collect()
}

fn first_env(vars: &[String]) -> Option<String> {
    vars.iter().find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
}

/// committed per store in .cake/config.hjson, shared by everyone using the store
#[derive(Deserialize, Default)]
struct StoreConfig {
    #[serde(default)]
    aliases: BTreeMap<String, String>,
}

impl Config {
    /// default aliases, then the user's, then the store's
    pub fn aliases(&self, store: &std::path::Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        let mut aliases: BTreeMap<String, String> = [
            ("review", "ls -t review --status todo,doing"),
            ("ask", "ls -t ask"),
            ("ideas", "ls --status none --childless"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        aliases.extend(self.aliases.clone());
        let file = store.join("config.hjson");
        if file.is_file() {
            let store_config: StoreConfig = deser_hjson::from_str(&fs::read_to_string(&file)?)
                .map_err(|e| format!("{}: {}", file.display(), e))?;
            aliases.extend(store_config.aliases);
        }
        Ok(aliases)
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
            .or_else(|| crate::git::user_name(store))
            .or_else(|| std::env::var("USER").ok())
            .unwrap_or("unknown".to_string())
    }

    /// creator of new items: CAKE_AUTHOR, else the agent, else git user.name, else $USER
    pub fn author(&self, store: &std::path::Path) -> String {
        first_env(&["CAKE_AUTHOR".to_string()])
            .or_else(|| self.agent())
            .or_else(|| crate::git::user_name(store))
            .or_else(|| std::env::var("USER").ok())
            .unwrap_or("unknown".to_string())
    }

    pub fn new() -> Result<Config, Box<dyn Error>> {
        if let Ok(serialized) = std::fs::read_to_string(util::config_file()?) {
            Ok(deser_hjson::from_str::<Config>(&serialized)?)
        } else {
            // default settings on linux at ~/.config/cake
            Ok(Config {
                dirty: true,
                editor: std::env::var("EDITOR").unwrap_or("vim".to_string()),
                disable_git: false,
                git_branch: default_git_branch(),
                git_remote_name: "origin".to_string(),
                agent_env: default_agent_env(),
                session_env: default_session_env(),
                aliases: BTreeMap::new(),
                status: default_status(),
                style: default_style(),
                scrollview_threshold: 30,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Sandbox;

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
