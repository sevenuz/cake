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
    pub animations: bool,
    pub usage_tracking: bool,
    pub score: crate::store::ScoreWeights,
    pub footprint: crate::footprint::Footprint,
}

impl Default for Config {
    fn default() -> Self {
        Config {
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
            animations: true,
            usage_tracking: true,
            score: Default::default(),
            footprint: Default::default(),
        }
    }
}

/// the defaults with explanations; must parse to Config::default()
pub const TEMPLATE: &str = r##"// cake configuration (hjson: comments, quotes optional).
// Remove a line to get its default back. config.default.hjson next to this
// file always holds the current defaults with these comments.
{
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
  // cake status: mine, others, ready, today, tag:<name>
  status: ["mine", "others", "ready", "today", "tag:review"]
  // own commands, may override built-ins for you (not for agents):
  // review: "ls -t review", ls: "ls --mine"
  aliases: {}
  // characters: auto (ascii without a UTF-8 locale), unicode, ascii;
  // env CAKE_STYLE wins. Colors: theme.hjson, off with NO_COLOR.
  style: auto
  // lines from which cake show opens a pager, -1 never
  scrollview_threshold: 30
  // the cake at the end of the morning form; false: shown without animation
  animations: true

  // tokens per item from agents (cake usage); false: the Claude hook does nothing
  usage_tracking: true

  // order of ls/ready and the morning form: higher first
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

  // Rough estimate of energy, CO2e and water behind the tokens of an item,
  // leaning high. Real values depend on model, hardware, batching, data
  // center and grid; read it as an order of magnitude.
  footprint: {
    // Wh per 1000 tokens for a large model, GPU only:
    // - output: 0.1 to 2 Wh per 1000 output tokens depending on model size
    //   (Epoch AI 2025, as cited in https://arxiv.org/pdf/2603.06630);
    //   Llama 3.1 405B about 0.21 Wh for ~367 output tokens per query
    //   (ML.ENERGY, NeurIPS 2025, reviewed in Joule,
    //   https://www.cell.com/joule/fulltext/S2542-4351(26)00114-5).
    //   Frontier models are larger: the top of the range.
    // - input: about 1/8 of an output token; 800 more output tokens cost
    //   ~10x the base energy, 800 more input tokens ~1.2x (OPT-30B on A100,
    //   https://arxiv.org/html/2511.05597)
    // - cache_write: like input (the prefill happens once)
    // - cache_read: no measurement found; assumed 1/10 of input, like the
    //   price ratio. In long agent sessions this term dominates.
    wh_per_1k: {
      output: 2.0
      input: 0.25
      cache_write: 0.25
      cache_read: 0.025
    }
    // factor per model name part, the longest match wins, * otherwise.
    // Assumption: sizes relative to the largest model, no published numbers.
    model_factor: {
      "*": 1.0
      opus: 1.0
      sonnet: 0.5
      haiku: 0.2
    }
    // server around the GPUs (CPU, memory, network): the measurements above
    // are GPU only. Assumption.
    server_overhead: 1.3
    // data center overhead (power usage effectiveness); assumption between
    // hyperscale (~1.1) and older facilities
    pue: 1.2
    // grid carbon intensity: EU 2021, 226.4 g CO2 per kWh (Climate
    // Transparency Report 2022, EU profile,
    // https://www.climate-transparency.org/wp-content/uploads/2022/10/CT2022-EU-Web.pdf).
    // Where the model runs is unknown; set your own grid.
    co2_g_per_kwh: 226.4
    // water on site per kWh of IT energy (WUE): industry average about
    // 1.8 l/kWh, hyperscale down to ~0.2
    // (https://www.hcltech.com/blogs/understanding-water-use-data-centers)
    wue_l_per_kwh: 1.8
    // water at the power plants per kWh drawn: thermoelectric plants about
    // 2 l/kWh, wind and solar near 0 (same source); taken high
    ewif_l_per_kwh: 2.0
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
        assert!(text.contains("// grid carbon intensity") && text.contains("226.4"));
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
