mod commands;
mod config;
mod dash;
mod error;
mod footprint;
mod git;
mod item;
mod log;
mod markdown;
mod migrate;
mod form;
mod gantt;
mod insights;
mod out;
mod selector;
mod stats;
mod store;
mod stores;
mod style;
mod theme;
mod usage;
#[cfg(test)]
mod test_support;
mod util;
mod view;

use crate::store::Store;
use clap::{Args, CommandFactory, Parser, Subcommand};
use config::Config;
use item::Status;
use selector::Selector;
use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::{
    env::{args, current_dir},
    error::Error,
    path::PathBuf,
};
use ratatui::crossterm::style::Stylize;

/// Tasks, time tracking and notes as markdown files, synced with git.
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// store directory, default: nearest .cake, else the global store
    #[arg(long, value_name = "DIR", env = "CAKE_STORE")]
    pub store: Option<PathBuf>,

    /// use the global store, also inside a project
    #[arg(short, long, conflicts_with = "store")]
    pub global: bool,

    /// print debug output (stderr)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// print one JSON document on stdout, errors too; never opens an editor
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args)]
#[command(next_help_heading = "Selectors (combined with and)")]
pub struct SelectorArgs {
    /// item ids, space or comma separated
    #[arg(value_name = "IDS", num_args = 0..)]
    words: Vec<String>,

    /// `words` joined, set by `Commands::normalize`
    #[arg(skip)]
    ids: Option<String>,

    /// items with these children
    #[arg(short, long, value_name = "IDS")]
    children: Option<String>,

    /// items under these parents
    #[arg(short, long, value_name = "IDS")]
    parents: Option<String>,

    /// items with these tags, ~tag excludes
    #[arg(short, long)]
    tags: Option<String>,

    /// created more than this long ago, e.g. 2d, 1w3h
    #[arg(short, long, value_name = "DURATION")]
    before: Option<String>,

    /// created less than this long ago
    #[arg(short, long, value_name = "DURATION")]
    after: Option<String>,

    /// timer running
    #[arg(long)]
    started: bool,

    /// timer not running
    #[arg(long)]
    stopped: bool,

    /// also children (-r), parents (-rr) or both (-rrr)
    #[arg(short, long, action = clap::ArgAction::Count)]
    recursive: u8,

    /// combine selectors with or
    #[arg(long)]
    or: bool,

    /// text in title or body, case-insensitive
    #[arg(short, long, value_name = "TEXT")]
    search: Option<String>,

    /// todo|doing|done|dropped|none (notes)
    #[arg(long)]
    status: Option<String>,

    /// open, no open blockers
    #[arg(long)]
    ready: bool,

    /// open, with open blockers
    #[arg(long)]
    blocked: bool,

    /// created by, e.g. a git user name or claude-code
    #[arg(long, value_name = "NAME")]
    by: Option<String>,

    /// items without children, e.g. ideas not worked out yet
    #[arg(long)]
    childless: bool,

    /// items you created (git user.name or CAKE_AUTHOR)
    #[arg(long, conflicts_with = "by")]
    mine: bool,

    /// assigned to any of these (comma separated; me = you)
    #[arg(long, value_name = "WHO")]
    assignee: Option<String>,

    /// a free field has this value (a list: contains it), repeatable
    #[arg(long = "where", value_name = "KEY=VALUE")]
    where_fields: Vec<String>,

    /// a free field is set, repeatable
    #[arg(long = "has", value_name = "KEY")]
    has_fields: Vec<String>,
}

/// the default goal of the dir we run in, when it links to `root`; a goal
/// that is gone: the top level, with a warning
fn default_parent(root: &std::path::Path, store: &Store) -> Option<String> {
    let cwd = current_dir().ok()?;
    let (linked, parent) = stores::link_of(&stores::load().ok()?, &cwd)?;
    let same = |a: &std::path::Path| std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    if same(&linked) != same(root) {
        return None;
    }
    let parent = parent?;
    if store.get_item(&parent).is_none() {
        eprintln!("the goal {} of this dir is gone: added at the top level (cake --store … init --parent <id> to fix)", parent);
        return None;
    }
    Some(parent)
}

/// replaces id prefixes in `opt` (comma separated) by full ids
fn resolve_opt(store: &Store, opt: &mut Option<String>) -> Result<(), Box<dyn Error>> {
    if let Some(v) = opt {
        *v = store.resolve_list(v)?;
    }
    Ok(())
}

impl SelectorArgs {
    /// resolves id prefixes and --mine
    fn resolve(&mut self, store: &Store, user: &str) -> Result<(), Box<dyn Error>> {
        if self.mine {
            self.by = Some(user.to_string());
        }
        if let Some(a) = &self.assignee {
            let list: Vec<String> = a.split(',').map(|w| if w.trim() == "me" { user.to_string() } else { w.trim().to_string() }).collect();
            self.assignee = Some(list.join(","));
        }
        resolve_opt(store, &mut self.ids)?;
        resolve_opt(store, &mut self.children)?;
        resolve_opt(store, &mut self.parents)
    }

    fn selector(&self, ids: &Option<String>) -> Result<Selector, Box<dyn Error>> {
        Selector::new(
            ids,
            &self.children,
            &self.parents,
            &self.tags,
            &self.before,
            &self.after,
            &self.started,
            &self.stopped,
            &self.recursive,
            &self.or,
        )?
        .with_filters(&self.search, &self.status, self.ready, self.blocked, &self.by)
        .map(|mut s| {
            s.childless = self.childless;
            s.assignee = self.assignee.iter().flat_map(|a| a.split(',')).map(String::from).filter(|a| !a.is_empty()).collect();
            s.has = self.has_fields.clone();
            s.where_ = self.where_fields.iter().map(|kv| kv.split_once('=').map_or((kv.clone(), String::new()), |(k, v)| (k.trim().into(), v.trim().into()))).collect();
            s
        })
    }
}

#[derive(Subcommand)]
pub enum UsageAction {
    /// add tokens of one model to an item (any agent)
    Add {
        id: String,
        #[arg(long)]
        model: String,
        #[arg(long = "in", default_value_t = 0)]
        input: u64,
        #[arg(long, default_value_t = 0)]
        out: u64,
        #[arg(long, default_value_t = 0)]
        cache_read: u64,
        #[arg(long, default_value_t = 0)]
        cache_write: u64,
    },
    /// Claude Code Stop hook: hook JSON on stdin, attributes the session's
    /// new answers to the items whose timer ran and logs the session's time
    ClaudeHook {},
    /// log time with cake open (the form and dashboard call it)
    #[command(hide = true)]
    Log {
        /// unix seconds
        #[arg(long)]
        start: i64,
        #[arg(long)]
        end: i64,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// capture: a new item in the inbox (--task, --note to skip sorting);
    /// the editor opens without -m
    Add {
        /// custom id, e.g. merge-driver; default: a random one (kqejfn)
        ids: Option<String>,

        /// move these items under the new one
        #[arg(short, long, value_name = "IDS")]
        children: Option<String>,

        /// parent item
        #[arg(short, long, value_name = "ID")]
        parents: Option<String>,

        /// tags
        #[arg(short, long)]
        tags: Option<String>,

        /// content, - reads stdin; the first `# heading` is the title
        #[arg(short, long)]
        message: Option<String>,

        /// a task right away instead of the inbox (agents: subtasks)
        #[arg(long, visible_alias = "todo")]
        task: bool,

        /// a note right away (knowledge, docs, rules) instead of the inbox
        #[arg(long, conflicts_with = "task")]
        note: bool,

        /// who does it: me, ai, an agent or a person
        #[arg(long, value_name = "WHO")]
        assignee: Option<String>,

        /// who reviews it, default: the parent's owner, else you
        #[arg(long, value_name = "WHO")]
        owner: Option<String>,

        /// soft target date: 2026-11-30, 30.11., today, tomorrow, fri, +3d, 2w
        #[arg(long, value_name = "DATE")]
        due: Option<String>,

        /// hard limit date, same forms as --due
        #[arg(long, value_name = "DATE")]
        deadline: Option<String>,

        /// priority, 1 (high) .. 3 (low)
        #[arg(long, value_name = "N")]
        prio: Option<u64>,

        /// expected work: 2h30m, 45m, 1d
        #[arg(long, value_name = "DURATION")]
        est: Option<String>,

        /// items that must be closed first
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,
    },
    /// change items: body, dates, priority, who, tags, blockers, parent,
    /// comments; without options one item opens in the editor
    Edit {
        /// item ids, space or comma separated
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,

        /// replace the body, - reads stdin; frontmatter stays
        #[arg(short, long)]
        message: Option<String>,

        /// priority 1 (high) .. 3 (low), 0 clears
        #[arg(long, value_name = "N")]
        prio: Option<u64>,

        /// soft target date (2026-11-30, 30.11., fri, +3d, ...), - clears
        #[arg(long, value_name = "DATE")]
        due: Option<String>,

        /// hard limit date, - clears
        #[arg(long, value_name = "DATE")]
        deadline: Option<String>,

        /// add git commits of the work (comma separated), ~hash removes one
        #[arg(long, value_name = "HASHES")]
        commit: Option<String>,

        /// expected work (2h30m, 45m), - clears
        #[arg(long, value_name = "DURATION")]
        est: Option<String>,

        /// who does it (me, ai, a name), - clears
        #[arg(long, value_name = "WHO")]
        assignee: Option<String>,

        /// who reviews it, - clears
        #[arg(long, value_name = "WHO")]
        owner: Option<String>,

        /// labels: a,b adds, ~a removes
        #[arg(short, long, value_name = "TAGS")]
        tag: Option<String>,

        /// a free header field: KEY=VALUE (numbers and true/false stay typed), repeatable
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set_fields: Vec<String>,

        /// adds to a free field: numbers are summed, lists appended, repeatable
        #[arg(long = "add", value_name = "KEY=VALUE")]
        add_fields: Vec<String>,

        /// removes a free field, repeatable
        #[arg(long = "unset", value_name = "KEY")]
        unset_fields: Vec<String>,

        /// order: x,y must be closed first, ~x removes x
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,

        /// move under this item, - to the top level
        #[arg(short, long, value_name = "ID")]
        parent: Option<String>,

        /// append a comment, back to the assignee; - reads stdin
        #[arg(short, long, value_name = "TEXT")]
        comment: Option<String>,
    },
    /// show an item, or any markdown file
    Show {
        /// item id or file path
        target: String,
    },
    /// list items as a tree; done and dropped only with -a, --status or ids
    #[command(visible_alias("ls"))]
    List {
        #[command(flatten)]
        selector: SelectorArgs,

        /// also done and dropped items
        #[arg(short = 'A', long)]
        all: bool,

        /// all fields and the body, also in json
        #[arg(short, long)]
        long: bool,

        /// also the archived items (implies -A)
        #[arg(long)]
        archive: bool,
    },
    /// one line for prompts and tmux: your timers, others' timers, ready, today
    Status {},
    /// time (you, agents), tokens, footprint and done items for a period,
    /// per store and goal
    Stats {
        /// day, 7d (rolling), week, month or year
        #[arg(short, long, default_value = "day")]
        period: String,
        /// every store from stores.json, not only this one
        #[arg(short, long)]
        all: bool,
    },
    /// start the timer, sets status doing
    Start {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// stop the timer, default: your running items
    Stop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set the status: inbox, note, task, doing, review, done, dropped;
    /// -m appends (an answer, a reason, what changed); aliases: done, drop,
    /// do, note, review
    Set {
        #[arg(value_name = "STATUS")]
        to: String,

        #[command(flatten)]
        selector: SelectorArgs,

        /// appended to the body, - reads stdin
        #[arg(short, long)]
        message: Option<String>,

        /// git commits of the work (comma separated), e.g. on review
        #[arg(long, value_name = "HASHES")]
        commit: Option<String>,
    },
    /// the form (also plain `cake`): config `form` modules, or only these:
    /// news sort answer review mine dump; -g: all stores
    Form {
        modules: Vec<String>,
        /// the tree starts on this item (with module all)
        #[arg(long, value_name = "ID")]
        at: Option<String>,
    },
    /// move a store from the tag based workflow to inbox/task/review with
    /// assignees; shows the changes, --apply makes them
    #[command(hide = true)]
    Migrate {
        #[arg(long)]
        apply: bool,
    },
    /// tokens per item, with an energy, CO2e and water estimate in show
    #[command(hide = true)]
    Usage {
        #[command(subcommand)]
        action: UsageAction,
    },
    /// move done and dropped subtrees closed long ago to items/archive/;
    /// views skip them, `ls --archive` and `show` find them
    Archive {
        /// closed at least this long ago: 3m (months), 2w, 10d, 1y
        #[arg(long, default_value = "3m")]
        older: String,
        /// only list what would move
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// give an item a new id; children, blockers and [[links]] follow
    Rename {
        id: String,
        new: String,
    },
    /// move an item under a parent, or with its subtree to another store
    Mv {
        id: String,
        /// parent id, - detaches
        #[arg(required_unless_present = "to_store", conflicts_with = "to_store")]
        parent: Option<String>,
        /// store name from stores.json, `global`, or a path; top-level there
        #[arg(long, value_name = "STORE")]
        to_store: Option<String>,
    },
    /// remove items
    #[command(visible_alias("rm"))]
    Remove {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// fetch, merge and push the store
    Sync {
        /// first push, sets the upstream; default remote from config
        #[arg(long, value_name = "REMOTE", num_args = 0..=1, default_missing_value = "", group = "mode")]
        publish: Option<String>,

        /// commit the resolved merge and push
        #[arg(long = "continue", group = "mode")]
        cont: bool,

        /// abort the merge
        #[arg(long, group = "mode")]
        abort: bool,
    },
    /// create a store; in a git repo on its own branch, attached if it exists on the remote;
    /// with --store DIR a separate store used from here (listed in stores.json); an existing
    /// one is linked to this dir
    Init {
        /// with --store: what is captured here goes under this item (a goal)
        #[arg(long)]
        parent: Option<String>,
    },
    /// open the config file in the editor
    Config {},
}

impl Commands {
    /// resolves id prefixes in all arguments that take ids
    fn resolve_ids(&mut self, store: &Store, user: &str) -> Result<(), Box<dyn Error>> {
        let one = |id: &mut String| -> Result<(), Box<dyn Error>> {
            *id = store.resolve(id)?;
            Ok(())
        };
        // "a,~b": resolve the ids, keep the ~
        let marked = |list: &mut String| -> Result<(), Box<dyn Error>> {
            let parts: Result<Vec<String>, Box<dyn Error>> = list
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .map(|p| match p.strip_prefix('~') {
                    Some(r) => Ok(format!("~{}", store.resolve(r)?)),
                    None => store.resolve(p),
                })
                .collect();
            *list = parts?.join(",");
            Ok(())
        };
        match self {
            Commands::Add { children, parents, blocked_by, .. } => {
                resolve_opt(store, children)?;
                if parents.as_deref() != Some("-") {
                    resolve_opt(store, parents)?;
                }
                resolve_opt(store, blocked_by)?;
            }
            Commands::Edit { ids, blocked_by, parent, .. } => {
                for id in ids.iter_mut() {
                    one(id)?;
                }
                if let Some(b) = blocked_by {
                    marked(b)?;
                }
                if let Some(p) = parent.as_mut().filter(|p| *p != "-") {
                    one(p)?;
                }
            }
            Commands::Show { target }
                if store.resolve(target).map_or(false, |t| store.get_item(&t).is_some() || store.is_archived(&t)) =>
            {
                one(target)?
            }
            Commands::Usage { action: UsageAction::Add { id, .. } } => one(id)?,
            Commands::Rename { id, .. } => one(id)?,
            Commands::Mv { id, parent, .. } => {
                one(id)?;
                if let Some(p) = parent.as_mut().filter(|p| *p != "-") {
                    one(p)?;
                }
            }
            Commands::Remove { selector }
            | Commands::Start { selector }
            | Commands::Stop { selector }
            | Commands::Set { selector, .. }
            | Commands::List { selector, .. } => selector.resolve(store, user)?,
            _ => (),
        }
        Ok(())
    }

    /// joins positional ids (space or comma separated)
    fn normalize(&mut self) {
        let join = |w: &[String]| (!w.is_empty()).then(|| w.join(","));
        match self {
            Commands::Edit { ids, .. } => {
                *ids = ids.iter().flat_map(|i| i.split(',')).map(|i| i.trim().to_string()).filter(|i| !i.is_empty()).collect();
            }
            Commands::Remove { selector }
            | Commands::Start { selector }
            | Commands::Stop { selector }
            | Commands::Set { selector, .. }
            | Commands::List { selector, .. } => {
                let w = std::mem::take(&mut selector.words);
                selector.ids = join(&w);
            }
            _ => (),
        }
    }

    /// true for commands that change items
    fn writes(&self) -> bool {
        matches!(
            self,
            Commands::Add { .. }
                | Commands::Edit { .. }
                | Commands::Remove { .. }
                | Commands::Set { .. }
                | Commands::Start { .. }
                | Commands::Stop { .. }
                | Commands::Mv { .. }
                | Commands::Rename { .. }
                | Commands::Archive { dry_run: false, .. }
                | Commands::Usage { .. }
                | Commands::Migrate { apply: true }
        )
    }
}

/// prints an error from `run`: json on stdout with `--json`, else text on stderr
pub fn print_error(e: &Box<dyn Error>) {
    if out::json() {
        let json = serde_json::json!({
            "v": 1,
            "error": { "code": error::code(e.as_ref()), "message": e.to_string() },
        });
        println!("{}", json);
    } else {
        eprintln!("That wasn't working o.0 \n {}", e.to_string().red().italic());
    }
}

/// fields of ls/ready --json without -l
const BRIEF: &[&str] = &[
    "id", "title", "status", "tags", "parent", "children", "blocked_by", "blocks", "ready", "score", "due",
    "deadline", "priority", "estimate", "running", "by", "assignee", "owner",
];

fn print_report(store: &Store, report: commands::Report) {
    let json = match report {
        commands::Report::None => serde_json::json!({ "v": 1 }),
        commands::Report::Items(ids) => serde_json::json!({
            "v": 1,
            "items": ids.iter().filter_map(|id| store.item_json(id)).collect::<Vec<_>>(),
        }),
        // ls and ready: what an agent needs to choose, -l or show for the rest
        commands::Report::Brief(ids) => serde_json::json!({
            "v": 1,
            "items": ids
                .iter()
                .filter_map(|id| store.item_json(id))
                .map(|mut i| {
                    i.as_object_mut().unwrap().retain(|k, _| BRIEF.contains(&k.as_str()));
                    i
                })
                .collect::<Vec<_>>(),
        }),
        commands::Report::Removed(ids) => serde_json::json!({ "v": 1, "removed": ids }),
        commands::Report::Value(serde_json::Value::Object(mut m)) => {
            m.insert("v".into(), 1.into());
            serde_json::Value::Object(m)
        }
        commands::Report::Value(v) => serde_json::json!({ "v": 1, "value": v }),
    };
    println!("{}", json);
}

/// -m value, "-" reads stdin; empty content is refused, e.g. from a pipe
/// whose writer died
fn read_message(message: &Option<String>) -> Result<Option<String>, Box<dyn Error>> {
    let text = match message.as_deref() {
        Some("-") => std::io::read_to_string(std::io::stdin())?,
        Some(m) => m.to_string(),
        None => return Ok(None),
    };
    if text.trim().is_empty() {
        return Err(error::invalid("Empty content, nothing written."));
    }
    Ok(Some(text))
}

/// one-line commit subject from the command line, at most 72 chars plus the agent tag
fn commit_message(args: &[String], agent: Option<String>) -> String {
    let line = format!("cake {}", args.join(" ").split_whitespace().collect::<Vec<_>>().join(" "));
    let mut message: String = line.chars().take(72).collect();
    if message.len() < line.len() {
        message.push('…');
    }
    if let Some(agent) = agent {
        message += &format!(" [{}]", agent);
    }
    message
}

/// store root before parsing, from --store, -g, CAKE_STORE or the current dir
fn guess_root(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--store" => return Ok(it.next().map(PathBuf::from).unwrap_or_default()),
            "-g" | "--global" => return util::global_store(),
            _ if a.starts_with("--store=") => return Ok(PathBuf::from(&a[8..])),
            _ if !a.starts_with('-') => break,
            _ => (),
        }
    }
    match std::env::var("CAKE_STORE") {
        Ok(s) if !s.is_empty() => Ok(PathBuf::from(s)),
        _ => util::find_store(&current_dir()?),
    }
}

/// replaces an alias in the command position with its command line, once,
/// so an alias may override a built-in command: ls = "ls --status todo"
/// an alias starting with `!` is a shell command (like git): its line
/// and the remaining arguments, run with sh in the current directory
fn shell_alias(args: &[String], aliases: &BTreeMap<String, String>) -> Option<(String, Vec<String>)> {
    let mut i = 1;
    while i < args.len() && args[i].starts_with('-') {
        i += if args[i] == "--store" { 2 } else { 1 };
    }
    let line = aliases.get(args.get(i)?)?.strip_prefix('!')?;
    Some((line.trim().to_string(), args[i + 1..].to_vec()))
}

fn expand_alias(args: &[String], aliases: &BTreeMap<String, String>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut i = 1;
    while i < args.len() && args[i].starts_with('-') {
        i += if args[i] == "--store" { 2 } else { 1 };
    }
    let Some(line) = args.get(i).and_then(|a| aliases.get(a)) else {
        return Ok(args.to_vec());
    };
    let words = shlex::split(line).ok_or_else(|| error::invalid(format!("alias {}: bad quoting", args[i])))?;
    Ok([&args[..i], &words[..], &args[i + 1..]].concat())
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();
    out::set_json(args.iter().any(|a| a == "--json")); // for errors before parsing
    let config = Config::new()?;
    style::init(&config.style);
    theme::init()?;
    footprint::init(config.footprint.clone());
    store::init_weights(config.score.clone());
    let mut aliases = config.aliases(&guess_root(&args)?)?;
    // agents and the form get the documented built-ins
    if config.agent().is_some() || std::env::var_os(form::PLAIN_ENV).is_some() {
        let builtin: Vec<String> = Cli::command()
            .get_subcommands()
            .flat_map(|c| std::iter::once(c.get_name().to_string()).chain(c.get_all_aliases().map(String::from)))
            .collect();
        aliases.retain(|k, _| !builtin.contains(k));
    }
    if let Some((line, rest)) = shell_alias(&args, &aliases) {
        // "$@" passes the remaining arguments on
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{} \"$@\"", line))
            .arg("cake")
            .args(rest)
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
    let alias_help: String = aliases
        .iter()
        .map(|(k, v)| format!("  {:<8} {}\n", k, v))
        .collect();
    let parsed = Cli::command()
        .after_help(format!("Aliases (config, store .cake/config.hjson):\n{}", alias_help))
        .try_get_matches_from(expand_alias(&args, &aliases)?)
        .and_then(|m| <Cli as clap::FromArgMatches>::from_arg_matches(&m));
    let mut cli = match parsed {
        Ok(cli) => cli,
        Err(e) if args.iter().any(|a| a == "--json") && e.use_stderr() => {
            out::set_json(true);
            return Err(error::invalid(e.to_string().trim()));
        }
        Err(e) => e.exit(),
    };
    out::set_json(cli.json);
    if let Some(c) = cli.command.as_mut() {
        c.normalize();
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let debug = |s: &str| match cli.debug {
        0 => (),
        _ => eprintln!("{}", s.yellow()),
    };
    debug(&format!("debug mode is on."));

    debug(&format!("{:?}", args));

    let root = match &cli.store {
        Some(p) => p.to_owned(),
        None if cli.global => util::global_store()?,
        None => util::find_store(&current_dir()?)?,
    };
    debug(&format!("Store: {}", root.display()));
    // project stores join the list on first use
    if root.file_name().is_some_and(|n| n == util::STORE_DIR) && root.join("items").is_dir() {
        let name = root.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string());
        stores::register(&root, name.as_deref().unwrap_or("store"), &[])?;
    }

    // the form runs every change as its own cake call: no lock while waiting
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal() && !out::json();
    match &cli.command {
        Some(Commands::Form { .. }) if !interactive => {
            return Err(error::invalid("the form needs a terminal and no --json."))
        }
        // bare cake: the dashboard; the form is one key away
        None if interactive => return dash::run(&root, &config.user(&root)),
        Some(Commands::Form { .. }) if interactive => {
            // named modules end with the last one; the configured run ends
            // in the ALL tree
            let (modules, last_tab, at) = match &cli.command {
                Some(Commands::Form { modules, at }) if !modules.is_empty() => (modules.clone(), false, at.clone()),
                Some(Commands::Form { at, .. }) => (config.form.clone(), true, at.clone()),
                _ => (config.form.clone(), true, None),
            };
            let aliases = config.aliases(&root)?;
            let agent_commands = ["triage", "work"].iter().filter(|c| aliases.contains_key(**c)).map(|c| c.to_string()).collect();
            let options = form::Options { modules, all: cli.global, width: config.form_width, keys: config.keys.clone(), tree_depth: config.tree_depth, sort_who: config.sort_who.clone(), agent_commands, last_tab, at };
            return form::run(&root, &config.user(&root), options);
        }
        _ => (),
    }

    // editors and stdin are read before taking the lock, so other commands are
    // not blocked meanwhile and `cake show x --json | … | cake edit x -m -` works
    let mut message = None;
    let mut comment = None;
    let mut edited = None;
    match &cli.command {
        Some(Commands::Add { message: m, .. }) => {
            message = read_message(m)?;
            if message.is_none() && std::io::stdin().is_terminal() && !out::json() {
                message = Some(util::input_from_external_editor(&config.editor, None)?);
            }
        }
        Some(Commands::Edit {
            ids,
            message: None,
            prio: None,
            due: None,
            deadline: None,
            commit: None,
            est: None,
            assignee: None,
            owner: None,
            tag: None,
            blocked_by: None,
            parent: None,
            comment: None,
            set_fields,
            add_fields,
            unset_fields,
        }) if ids.len() == 1 && set_fields.is_empty() && add_fields.is_empty() && unset_fields.is_empty() => edited = Some(commands::edit_in_editor(&config, &root, &ids[0])?),
        // stdin may come from another cake command that needs the lock
        Some(Commands::Edit { message: m, comment: c, .. }) => {
            message = read_message(m)?;
            comment = read_message(c)?;
        }
        Some(Commands::Set { message: m, .. }) => message = read_message(m)?,
        _ => (),
    }

    let exclusive = cli.command.as_ref().map_or(false, |c| {
        c.writes() || matches!(c, Commands::Sync { .. })
    });
    let mut lock = util::lock_file(&root)?;
    let _guard = match lock.as_mut() {
        Some(l) => Some(util::lock(l, exclusive)?),
        None => None,
    };

    // before loading: conflicted items may not parse
    if let Some(Commands::Sync {
        publish,
        cont,
        abort,
    }) = &cli.command
    {
        let report = commands::sync(debug, &config, &root, publish, *cont, *abort)?;
        if out::json() {
            print_report(&Store::new_except(&root, &[])?, report);
        }
        return Ok(());
    }
    // during an unresolved merge: reads skip conflicted items, writes are refused
    let mut conflicted = vec![];
    if !config.disable_git && git::is_store_repo(&root) && git::merge_in_progress(&root)? {
        conflicted = git::conflicted_files(&root)?;
        let msg = format!(
            "store has an unresolved merge, conflicted: {}. Resolve, then `cake sync --continue` (or `--abort`).",
            conflicted.join(", ")
        );
        if cli.command.as_ref().map_or(false, Commands::writes) {
            return Err(error::conflict(msg));
        }
        eprintln!("{}", msg.yellow());
    }
    let skip: Vec<PathBuf> = conflicted.iter().map(|f| root.join(f)).collect();
    let mut store = Store::new_except(&root, &skip)?;
    if let Some(command) = cli.command.as_mut() {
        command.resolve_ids(&store, &config.user(&root))?;
        // an archived id named, or the archive asked for
        if store.archive_hit() || matches!(command, Commands::List { archive: true, .. }) {
            store.include_archive();
        }
    }

    let report = match &cli.command {
        Some(Commands::Add {
            ids,
            children,
            parents,
            tags,
            task,
            note,
            assignee,
            owner,
            due,
            deadline,
            prio,
            est,
            blocked_by,
            ..
        }) => {
            // -p - : the top level; no -p: the goal of the linked dir
            let parents = match parents.as_deref() {
                Some("-") => None,
                Some(_) => parents.clone(),
                None if cli.global => None,
                None => default_parent(&root, &store),
            };
            let report = commands::add(
            &debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, &parents, tags, &None, &None, &false, &false, &0, &false,
            )?,
            message,
            commands::AddOptions {
                status: if *task {
                    Some(Status::Task)
                } else if *note {
                    None
                } else {
                    Some(Status::Inbox)
                },
                assignee: assignee.as_deref().map(|w| config.who(&root, w)).transpose()?.flatten(),
                owner: owner.as_deref().map(|w| config.who(&root, w)).transpose()?.flatten(),
                user: config.user(&root),
                due: due.to_owned(),
                deadline: deadline.to_owned(),
                priority: *prio,
                estimate: est.to_owned(),
                blocked_by: util::split_comma_cleanup(blocked_by.to_owned().unwrap_or_default()),
            },
        )?;
            if let (Some(p), commands::Report::Items(ids)) = (&parents, &report) {
                if !out::json() {
                    eprintln!("{} → {}", ids.join(", "), p);
                }
            }
            report
        }
        Some(Commands::Edit { ids, prio, due, deadline, est, assignee, owner, tag, blocked_by, parent, set_fields, add_fields, unset_fields, .. }) => {
            let pair = |kv: &String| -> Result<(String, serde_yaml_ng::Value), Box<dyn Error>> {
                let (k, v) = kv.split_once('=').ok_or_else(|| error::invalid(format!("'{}': use KEY=VALUE", kv)))?;
                Ok((k.trim().to_string(), item::field_value(v.trim())))
            };
            let sets = set_fields.iter().map(pair).collect::<Result<Vec<_>, _>>()?;
            let adds = add_fields.iter().map(pair).collect::<Result<Vec<_>, _>>()?;
            let fields = edited.is_some()
                || message.is_some()
                || est.is_some()
                || prio.is_some()
                || due.is_some()
                || deadline.is_some()
                || assignee.is_some()
                || owner.is_some();
            let assignee = assignee.as_deref().map(|w| config.who(&root, w)).transpose()?;
            let owner = owner.as_deref().map(|w| config.who(&root, w)).transpose()?;
            for id in ids {
                store.get_item(id).ok_or_else(|| error::not_found(id))?;
                if fields {
                    commands::edit(
                        &debug,
                        &config,
                        &mut store,
                        id,
                        commands::EditOptions {
                            edited: edited.clone(),
                            body: message.clone(),
                            priority: *prio,
                            due: due.to_owned(),
                            deadline: deadline.to_owned(),
                            estimate: est.to_owned(),
                            assignee: assignee.clone(),
                            owner: owner.clone(),
                        },
                    )?;
                }
                {
                    let item = store.get_item_mut(id).unwrap();
                    for (k, v) in &sets {
                        item.set_field(k, v.clone()).map_err(error::invalid)?;
                    }
                    for (k, v) in &adds {
                        item.add_field(k, v.clone()).map_err(error::invalid)?;
                    }
                    for k in unset_fields {
                        item.unset_field(k).map_err(error::invalid)?;
                    }
                }
                if let Some(t) = tag {
                    store
                        .get_item_mut(id)
                        .unwrap()
                        .append_tags(util::split_comma_tags(t.clone()))
                        .remove_tags(util::split_comma_exclude_tags(t.clone()));
                }
                if let Some(b) = blocked_by {
                    let (rm, add): (Vec<&str>, Vec<&str>) = b.split(',').filter(|x| !x.is_empty()).partition(|x| x.starts_with('~'));
                    let rm: Vec<String> = rm.iter().map(|x| x[1..].to_string()).collect();
                    let add: Vec<String> = add.iter().map(|x| x.to_string()).collect();
                    if !add.is_empty() {
                        store.block(id, &add)?;
                    }
                    if !rm.is_empty() {
                        store.unblock(id, &rm)?;
                    }
                }
                if let Some(p) = parent {
                    store.move_item(id, (p != "-").then_some(p.as_str()))?;
                }
                if let Some(c) = &comment {
                    commands::comment(&mut store, id, &config.author(&root), c)?;
                }
            }
            commands::Report::Items(ids.clone())
        }
        Some(Commands::Remove { selector }) => {
            commands::remove(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Start { selector }) => {
            if config.agent().is_some() {
                if let Some(msg) = commands::reviews_full(&store, &Config::store(&root)?) {
                    return Err(error::coded("waiting_for_review", msg));
                }
            }
            commands::start(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Stop { selector }) => {
            commands::stop(debug, &config, &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Mv { id, parent: Some(parent), .. }) => commands::mv(debug, &mut store, id, parent)?,
        Some(Commands::Mv { id, to_store: Some(target), .. }) => commands::mv_to_store(
            debug,
            &config,
            &mut store,
            id,
            &stores::resolve(&stores::load()?, target)?,
            &commit_message(&args[1..], config.agent()),
        )?,
        Some(Commands::Mv { .. }) => unreachable!("clap requires a parent or --to-store"),
        Some(Commands::Rename { id, new }) => {
            let report = commands::rename(debug, &mut store, id, new)?;
            if let Err(e) = stores::rename_parent(&root, id, new) {
                eprintln!("stores.json: links to {} keep the old id: {}", id, e);
            }
            report
        }
        Some(Commands::Archive { older, dry_run }) => commands::archive(debug, &mut store, older, *dry_run)?,
        Some(Commands::Set { to, selector, .. }) => {
            let selector = selector.selector(&selector.ids)?;
            let name = if to == "drop" { "dropped" } else { to.as_str() };
            match Status::parse(name).map_err(error::invalid)? {
                None => commands::to_note(&mut store, selector)?,
                Some(Status::Task) => {
                    let report = commands::set_status(debug, &mut store, selector, Status::Task, message)?;
                    commands::default_owner(&mut store, &report, &config.user(&root));
                    report
                }
                Some(st) => commands::set_status(debug, &mut store, selector, st, message)?,
            }
        }
        Some(Commands::List { selector, long, all, archive }) => {
            let all = &(*all || *archive);
            let mut selector = selector.selector(&selector.ids)?;
            if selector.ready {
                // an agent sees its own tasks and those for any agent, and
                // none while the reviews are full
                if let Some(agent) = config.agent() {
                    if selector.assignee.is_empty() {
                        selector.assignee = vec![agent, "ai".into()];
                    }
                }
                if let Some(msg) = config.agent().and(commands::reviews_full(&store, &Config::store(&root)?)) {
                    say!("{}", msg);
                    print_report(&store, commands::Report::Brief(vec![]));
                    return Ok(());
                }
            } else {
                selector.rchildren = true;
            }
            commands::list(debug, &config, &mut store, selector, *long, *all)?
        }
        Some(Commands::Status {}) => commands::status(&config, &store)?,
        Some(Commands::Stats { period, all }) => commands::stats(&config, &store, period, *all)?,
        Some(Commands::Show { target }) => commands::show(debug, &config, &store, target)?,
        Some(Commands::Init { parent }) => commands::init(debug, &config, cli.global, cli.store.as_deref(), parent.as_deref(), &store)?,
        Some(Commands::Config {}) => commands::config(debug, &config)?,
        Some(Commands::Sync { .. }) => unreachable!("handled before loading the store"),
        Some(Commands::Form { .. }) => unreachable!("handled before the lock"),
        Some(Commands::Migrate { apply }) => crate::migrate::run(&mut store, &config.user(&root), *apply)?,
        Some(Commands::Usage { action: UsageAction::Add { id, model, input, out, cache_read, cache_write } }) => {
            let u = item::Usage { input: *input, output: *out, cache_read: *cache_read, cache_write: *cache_write };
            store.get_item_mut(id).ok_or_else(|| error::not_found(id))?.add_usage(model, u);
            commands::Report::Items(vec![id.to_owned()])
        }
        Some(Commands::Usage { action: UsageAction::ClaudeHook {} }) => {
            if config.usage_tracking {
                usage::claude_hook(&mut store, Some(&config.user(&root)))?;
            }
            commands::Report::None
        }
        Some(Commands::Usage { action: UsageAction::Log { start, end } }) => {
            if end > start {
                log::append(&root, &[log::Line::new(*start, *end, &config.user(&root), "cake")])?;
            }
            commands::Report::None
        }
        None => {
            if !out::json() {
                Cli::command().print_help()?;
            }
            commands::Report::None
        }
    };

    // --commit on set and edit: the hashes go to every changed item
    let commit = match &cli.command {
        Some(Commands::Set { commit, .. }) | Some(Commands::Edit { commit, .. }) => commit.clone(),
        _ => None,
    };
    if let (Some(c), commands::Report::Items(ids) | commands::Report::Brief(ids)) = (commit, &report) {
        let hashes: Vec<String> = c.split(',').map(|h| h.trim().to_string()).filter(|h| !h.is_empty()).collect();
        for id in ids {
            if let Some(i) = store.get_item_mut(id) {
                i.add_commits(&hashes);
            }
        }
    }
    if cli.command.as_ref().is_some_and(Commands::writes) {
        usage::fold(&mut store)?;
        store.settle();
    }
    if let Some(session) = config.session() {
        store.add_session(&session);
    }
    let mut changed = store.save()?;
    // the session log rides along with the next commit of items: a commit
    // per hook call (every answer) would flood the history
    let log_dir = root.join("log");
    if !changed.is_empty() && log_dir.is_dir() {
        changed.push(log_dir);
    }
    if !config.disable_git && git::is_store_repo(&root) {
        let message = commit_message(&args[1..], config.agent());
        debug(&format!("git commit {:?}: {}", changed, message));
        git::commit(&root, &changed, &message)?;
    }
    if out::json() {
        print_report(&store, report);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn commit_message_is_one_short_line_with_agent_tag() {
        let args: Vec<String> = ["add", "-m", "# Title\n\nlong body"].iter().map(|s| s.to_string()).collect();
        assert_eq!(commit_message(&args, None), "cake add -m # Title long body");
        let long = vec!["x".repeat(100)];
        let m = commit_message(&long, Some("claude-code".into()));
        assert!(m.ends_with("… [claude-code]") && m.chars().count() == 72 + 1 + 14, "{}", m);
    }

    #[test]
    fn aliases_expand_in_command_position_with_extra_args() {
        let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let aliases: BTreeMap<String, String> =
            [("review".to_string(), "ls -t review -s 'two words'".to_string())].into();
        assert_eq!(
            expand_alias(&s(&["cake", "--json", "--store", "x", "review", "-p", "a"]), &aliases).unwrap(),
            s(&["cake", "--json", "--store", "x", "ls", "-t", "review", "-s", "two words", "-p", "a"])
        );
        assert_eq!(expand_alias(&s(&["cake", "ls", "review"]), &aliases).unwrap(), s(&["cake", "ls", "review"]));
        let own_ls: BTreeMap<String, String> = [("ls".to_string(), "ls --status todo".to_string())].into();
        assert_eq!(
            expand_alias(&s(&["cake", "ls", "-t", "x"]), &own_ls).unwrap(),
            s(&["cake", "ls", "--status", "todo", "-t", "x"]),
            "an alias may override a built-in, expanded once"
        );
    }

    #[test]
    fn bang_aliases_are_shell_commands_with_the_remaining_args() {
        let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let aliases: BTreeMap<String, String> =
            [("triage".to_string(), "!claude -p 'x'".to_string()), ("rev".to_string(), "ls".to_string())].into();
        assert_eq!(
            shell_alias(&s(&["cake", "-g", "triage", "a", "b"]), &aliases),
            Some(("claude -p 'x'".to_string(), s(&["a", "b"])))
        );
        assert_eq!(shell_alias(&s(&["cake", "rev"]), &aliases), None);
    }

    #[test]
    fn global_flag_parses_and_conflicts_with_store() {
        assert!(Cli::try_parse_from(["cake", "-g", "init"]).unwrap().global);
        assert!(Cli::try_parse_from(["cake", "-g", "--store", "x", "ls"]).is_err());
    }

    #[test]
    fn sync_publish_takes_an_optional_remote_and_modes_conflict() {
        let publish = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
            Some(Commands::Sync { publish, .. }) => publish,
            _ => panic!("expected sync"),
        };
        assert_eq!(publish(&["cake", "sync"]), None);
        assert_eq!(publish(&["cake", "sync", "--publish"]).as_deref(), Some(""));
        assert_eq!(publish(&["cake", "sync", "--publish", "up"]).as_deref(), Some("up"));
        assert!(Cli::try_parse_from(["cake", "sync", "--continue", "--abort"]).is_err());
    }

    #[test]
    fn only_item_changing_commands_write() {
        let writes = |args: &[&str]| Cli::try_parse_from(args).unwrap().command.unwrap().writes();
        assert!(writes(&["cake", "add", "-m", "x"]));
        assert!(writes(&["cake", "start", "abc"]));
        assert!(!writes(&["cake", "ls"]));
        assert!(!writes(&["cake", "show", "x.md"]));
    }

    #[test]
    fn set_takes_the_status_first_and_ids_space_or_comma_separated() {
        let parse = |args: &[&str]| {
            let mut cli = Cli::try_parse_from(args).unwrap();
            cli.command.as_mut().unwrap().normalize();
            cli.command.unwrap()
        };
        match parse(&["cake", "set", "done", "a", "b,c", "-m", "x", "-t", "~z", "--or"]) {
            Commands::Set { to, selector, message, .. } => {
                assert_eq!(to, "done");
                assert_eq!(selector.ids.as_deref(), Some("a,b,c"));
                assert_eq!(selector.tags.as_deref(), Some("~z"));
                assert!(selector.or);
                assert_eq!(message.as_deref(), Some("x"));
            }
            _ => panic!("expected set"),
        }
        match parse(&["cake", "edit", "a", "b,c", "--tag", "x,~y", "--blocked-by", "q", "--comment", "hi"]) {
            Commands::Edit { ids, tag, blocked_by, comment, .. } => {
                assert_eq!(ids, vec!["a", "b", "c"]);
                assert_eq!((tag.as_deref(), blocked_by.as_deref(), comment.as_deref()), (Some("x,~y"), Some("q"), Some("hi")));
            }
            _ => panic!("expected edit"),
        }
    }
}
