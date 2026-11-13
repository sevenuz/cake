mod commands;
mod config;
mod error;
mod footprint;
mod git;
mod item;
mod markdown;
mod migrate;
mod form;
mod out;
mod selector;
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
    /// new answers to the items whose timer ran
    ClaudeHook {},
}

#[derive(Subcommand)]
pub enum Commands {
    /// capture: a new item in the inbox (--task, --note to skip sorting);
    /// the editor opens without -m
    Add {
        /// custom id, default: slug of the title (e.g. fix-merge-driver)
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

        /// items that must be closed first
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,
    },
    /// edit the item file in the editor, or change body/priority/dates directly
    Edit {
        id: String,

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

        /// who does it (me, ai, a name), - clears
        #[arg(long, value_name = "WHO")]
        assignee: Option<String>,

        /// who reviews it, - clears
        #[arg(long, value_name = "WHO")]
        owner: Option<String>,
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
    },
    /// one line for prompts and tmux: your timers, others' timers, ready, today
    Status {},
    /// list open items without open blockers, by priority and due
    Ready {
        #[command(flatten)]
        selector: SelectorArgs,

        /// all fields and the body, also in json
        #[arg(short, long)]
        long: bool,
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
    /// set status done, stops the timer; -m appends a closing note (e.g. an answer)
    Done {
        #[command(flatten)]
        selector: SelectorArgs,

        /// appended to the body, - reads stdin
        #[arg(short, long)]
        message: Option<String>,
    },
    /// set status dropped, stops the timer; -m appends the reason
    Drop {
        #[command(flatten)]
        selector: SelectorArgs,

        /// appended to the body, - reads stdin
        #[arg(short, long)]
        message: Option<String>,
    },
    /// make it a task (you become its owner); also reopens
    #[command(name = "do", visible_aliases(["task", "todo", "reopen"]))]
    Todo {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// keep it as a note: knowledge, idea, rule, nothing to do
    Note {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// finished: the owner checks it; -m appends what changed and how to try it
    Review {
        #[command(flatten)]
        selector: SelectorArgs,

        /// appended to the body, - reads stdin
        #[arg(short, long)]
        message: Option<String>,
    },
    /// who does it: cake assign [IDS] WHO (me, ai, an agent, a person, -)
    Assign {
        #[command(flatten)]
        selector: SelectorArgs,

        /// the last positional word
        #[arg(skip)]
        who: Option<String>,
    },
    /// append a comment and give the item back to its assignee
    Comment {
        id: String,

        /// the comment, - reads stdin
        #[arg(short, long)]
        message: Option<String>,
    },
    /// add tags, ~tag removes: cake tag [IDS] TAGS (the last word are the tags)
    Tag {
        #[command(flatten)]
        selector: SelectorArgs,

        /// tags to add, ~tag to remove: the last positional word
        #[arg(skip)]
        new_tags: Option<String>,
    },
    /// the form: config `form` modules in order; -g: all stores
    Morning {},
    /// run only these form modules: news sort answer review mine triage dump
    Form {
        modules: Vec<String>,
    },
    /// move a store from the tag based workflow to inbox/task/review with
    /// assignees; shows the changes, --apply makes them
    Migrate {
        #[arg(long)]
        apply: bool,
    },
    /// tokens per item, with an energy, CO2e and water estimate in show
    Usage {
        #[command(subcommand)]
        action: UsageAction,
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
    /// add items that must be closed first
    Block {
        id: String,
        #[arg(long, value_name = "IDS")]
        by: String,
    },
    /// remove blockers
    Unblock {
        id: String,
        #[arg(long, value_name = "IDS")]
        by: String,
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
    /// with --store DIR a separate store used from here (listed in stores.json)
    Init {},
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
        match self {
            Commands::Add { children, parents, blocked_by, .. } => {
                resolve_opt(store, children)?;
                resolve_opt(store, parents)?;
                resolve_opt(store, blocked_by)?;
            }
            Commands::Edit { id, .. } => one(id)?,
            Commands::Show { target } if store.resolve(target).map_or(false, |t| store.get_item(&t).is_some()) => {
                one(target)?
            }
            Commands::Usage { action: UsageAction::Add { id, .. } } => one(id)?,
            Commands::Mv { id, parent, .. } => {
                one(id)?;
                if let Some(p) = parent.as_mut().filter(|p| *p != "-") {
                    one(p)?;
                }
            }
            Commands::Block { id, by } | Commands::Unblock { id, by } => {
                one(id)?;
                *by = store.resolve_list(by)?;
            }
            // with one positional only, `tag` treats it as the tags
            Commands::Tag { selector, new_tags } if new_tags.is_none() => {
                resolve_opt(store, &mut selector.children)?;
                resolve_opt(store, &mut selector.parents)?;
            }
            Commands::Remove { selector }
            | Commands::Tag { selector, .. }
            | Commands::Start { selector }
            | Commands::Stop { selector }
            | Commands::Done { selector, .. }
            | Commands::Drop { selector, .. }
            | Commands::Todo { selector }
            | Commands::Note { selector }
            | Commands::Review { selector, .. }
            | Commands::Assign { selector, .. }
            | Commands::List { selector, .. }
            | Commands::Ready { selector, .. } => selector.resolve(store, user)?,
            Commands::Comment { id, .. } => one(id)?,
            _ => (),
        }
        Ok(())
    }

    /// joins positional ids; for tag the last word holds the tags
    fn normalize(&mut self) {
        let join = |w: &[String]| (!w.is_empty()).then(|| w.join(","));
        match self {
            Commands::Tag { selector, new_tags } => {
                let w = std::mem::take(&mut selector.words);
                if w.len() >= 2 {
                    *new_tags = Some(w[w.len() - 1].clone());
                    selector.ids = join(&w[..w.len() - 1]);
                } else {
                    selector.ids = join(&w);
                }
            }
            Commands::Assign { selector, who } => {
                let mut w = std::mem::take(&mut selector.words);
                *who = w.pop();
                selector.ids = join(&w);
            }
            Commands::Remove { selector }
            | Commands::Start { selector }
            | Commands::Stop { selector }
            | Commands::Done { selector, .. }
            | Commands::Drop { selector, .. }
            | Commands::Todo { selector }
            | Commands::Note { selector }
            | Commands::Review { selector, .. }
            | Commands::List { selector, .. }
            | Commands::Ready { selector, .. } => {
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
                | Commands::Tag { .. }
                | Commands::Start { .. }
                | Commands::Stop { .. }
                | Commands::Mv { .. }
                | Commands::Usage { .. }
                | Commands::Block { .. }
                | Commands::Unblock { .. }
                | Commands::Done { .. }
                | Commands::Drop { .. }
                | Commands::Todo { .. }
                | Commands::Note { .. }
                | Commands::Review { .. }
                | Commands::Assign { .. }
                | Commands::Comment { .. }
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
    "deadline", "priority", "running", "by", "assignee", "owner",
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
    // agents and the morning form get the documented built-ins
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
        Some(Commands::Morning {}) | Some(Commands::Form { .. }) if !interactive => {
            return Err(error::invalid("the form needs a terminal and no --json."))
        }
        Some(Commands::Morning {}) | Some(Commands::Form { .. }) | None if interactive => {
            let modules = match &cli.command {
                Some(Commands::Form { modules }) if !modules.is_empty() => modules.clone(),
                _ => config.form.clone(),
            };
            let triage = aliases.contains_key("triage");
            let options = form::Options { modules, all: cli.global, width: config.form_width, triage, keys: config.keys.clone() };
            return form::run(&root, &config.user(&root), options);
        }
        _ => (),
    }

    // editors and stdin are read before taking the lock, so other commands are
    // not blocked meanwhile and `cake show x --json | … | cake edit x -m -` works
    let mut message = None;
    let mut edited = None;
    match &cli.command {
        Some(Commands::Add { message: m, .. }) => {
            message = read_message(m)?;
            if message.is_none() && std::io::stdin().is_terminal() && !out::json() {
                message = Some(util::input_from_external_editor(&config.editor, None)?);
            }
        }
        Some(Commands::Edit {
            id,
            message: None,
            prio: None,
            due: None,
            deadline: None,
            assignee: None,
            owner: None,
        }) => edited = Some(commands::edit_in_editor(&config, &root, id)?),
        // stdin may come from another cake command that needs the lock
        Some(Commands::Edit { message: m, .. })
        | Some(Commands::Done { message: m, .. })
        | Some(Commands::Drop { message: m, .. })
        | Some(Commands::Review { message: m, .. })
        | Some(Commands::Comment { message: m, .. }) => message = read_message(m)?,
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
            blocked_by,
            ..
        }) => commands::add(
            &debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, parents, tags, &None, &None, &false, &false, &0, &false,
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
                blocked_by: util::split_comma_cleanup(blocked_by.to_owned().unwrap_or_default()),
            },
        )?,
        Some(Commands::Edit { id, prio, due, deadline, assignee, owner, .. }) => commands::edit(
            &debug,
            &config,
            &mut store,
            id,
            commands::EditOptions {
                edited,
                body: message,
                priority: *prio,
                due: due.to_owned(),
                deadline: deadline.to_owned(),
                assignee: assignee.as_deref().map(|w| config.who(&root, w)).transpose()?,
                owner: owner.as_deref().map(|w| config.who(&root, w)).transpose()?,
            },
        )?,
        Some(Commands::Remove { selector }) => {
            commands::remove(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Tag { selector, new_tags }) => {
            let mut i = &selector.ids;
            let mut nt = new_tags;
            // use ids for tags if only one option is set.
            if i.is_none() && nt.is_none() {
                return Err(error::invalid("You have to specify tags: cake tag [IDS] TAGS"));
            } else if nt.is_none() {
                nt = i;
                i = new_tags;
            }
            commands::tag(debug, &mut &mut store, selector.selector(i)?, nt)?
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
        Some(Commands::Block { id, by }) => commands::block(debug, &mut store, id, by, false)?,
        Some(Commands::Unblock { id, by }) => commands::block(debug, &mut store, id, by, true)?,
        Some(Commands::Ready { selector, long }) => {
            let mut selector = selector.selector(&selector.ids)?;
            selector.ready = true;
            // an agent sees its own tasks and those for any agent, and none
            // while the reviews are full
            if let Some(agent) = config.agent() {
                if selector.assignee.is_empty() {
                    selector.assignee = vec![agent, "ai".into()];
                }
            }
            match config.agent().and(commands::reviews_full(&store, &Config::store(&root)?)) {
                Some(msg) => {
                    say!("{}", msg);
                    commands::Report::Brief(vec![])
                }
                None => commands::list(debug, &config, &mut store, selector, *long, false)?,
            }
        }
        Some(Commands::Done { selector, .. }) => {
            commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Done, message)?
        }
        Some(Commands::Drop { selector, .. }) => {
            commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Dropped, message)?
        }
        Some(Commands::Todo { selector }) => {
            let report = commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Task, None)?;
            commands::default_owner(&mut store, &report, &config.user(&root));
            report
        }
        Some(Commands::Note { selector }) => commands::to_note(&mut store, selector.selector(&selector.ids)?)?,
        Some(Commands::Review { selector, .. }) => {
            commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Review, message)?
        }
        Some(Commands::Assign { selector, who }) => {
            let who = who.as_deref().ok_or_else(|| error::invalid("Who? cake assign [IDS] WHO"))?;
            commands::assign(&mut store, selector.selector(&selector.ids)?, config.who(&root, who)?)?
        }
        Some(Commands::Comment { id, .. }) => {
            let text = message.ok_or_else(|| error::invalid("No comment, use -m TEXT (- for stdin)."))?;
            commands::comment(&mut store, id, &config.user(&root), &text)?
        }
        Some(Commands::List { selector, long, all }) => {
            let mut selector = selector.selector(&selector.ids)?;
            selector.rchildren = true;
            commands::list(debug, &config, &mut store, selector, *long, *all)?
        }
        Some(Commands::Status {}) => commands::status(&config, &store)?,
        Some(Commands::Show { target }) => commands::show(debug, &config, &store, target)?,
        Some(Commands::Init {}) => commands::init(debug, &config, cli.global, cli.store.as_deref())?,
        Some(Commands::Config {}) => commands::config(debug, &config)?,
        Some(Commands::Sync { .. }) => unreachable!("handled before loading the store"),
        Some(Commands::Morning {}) | Some(Commands::Form { .. }) => unreachable!("handled before the lock"),
        Some(Commands::Migrate { apply }) => crate::migrate::run(&mut store, &config.user(&root), *apply)?,
        Some(Commands::Usage { action: UsageAction::Add { id, model, input, out, cache_read, cache_write } }) => {
            let u = item::Usage { input: *input, output: *out, cache_read: *cache_read, cache_write: *cache_write };
            store.get_item_mut(id).ok_or_else(|| error::not_found(id))?.add_usage(model, u);
            commands::Report::Items(vec![id.to_owned()])
        }
        Some(Commands::Usage { action: UsageAction::ClaudeHook {} }) => {
            if config.usage_tracking {
                usage::claude_hook(&mut store)?;
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

    if cli.command.as_ref().is_some_and(Commands::writes) {
        usage::fold(&mut store)?;
        store.settle();
    }
    if let Some(session) = config.session() {
        store.add_session(&session);
    }
    let changed = store.save()?;
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
    fn ids_may_be_space_separated_and_tag_takes_the_last_word() {
        let parse = |args: &[&str]| {
            let mut cli = Cli::try_parse_from(args).unwrap();
            cli.command.as_mut().unwrap().normalize();
            cli.command.unwrap()
        };
        match parse(&["cake", "done", "a", "b,c", "-m", "x"]) {
            Commands::Done { selector, .. } => assert_eq!(selector.ids.as_deref(), Some("a,b,c")),
            _ => panic!("expected done"),
        }
        match parse(&["cake", "tag", "a", "b", "x,~y"]) {
            Commands::Tag { selector, new_tags } => {
                assert_eq!(selector.ids.as_deref(), Some("a,b"));
                assert_eq!(new_tags.as_deref(), Some("x,~y"));
            }
            _ => panic!("expected tag"),
        }
        assert!(matches!(parse(&["cake", "do", "a"]), Commands::Todo { .. }));
        assert!(matches!(parse(&["cake", "todo", "a"]), Commands::Todo { .. }));
    }

    #[test]
    fn tag_parses_ids_before_new_tags_and_flattened_selector_flags() {
        let mut cli =
            Cli::try_parse_from(["cake", "tag", "a,b", "x,y", "-t", "~z", "-rr", "--or"]).unwrap();
        cli.command.as_mut().unwrap().normalize();
        match cli.command {
            Some(Commands::Tag { selector, new_tags }) => {
                assert_eq!(selector.ids.as_deref(), Some("a,b"));
                assert_eq!(new_tags.as_deref(), Some("x,y"));
                assert_eq!(selector.tags.as_deref(), Some("~z"));
                assert_eq!(selector.recursive, 2);
                assert!(selector.or);
            }
            _ => panic!("expected tag command"),
        }
    }
}
