mod commands;
mod config;
mod error;
mod git;
mod item;
mod out;
mod selector;
mod store;
#[cfg(test)]
mod test_support;
mod util;
mod view;

use crate::store::Store;
use clap::{Args, CommandFactory, Parser, Subcommand};
use config::Config;
use item::Status;
use selector::Selector;
use std::{
    env::{args, current_dir},
    error::Error,
    path::PathBuf,
};
use termimad::crossterm::style::Stylize;

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
    /// item ids, comma separated
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
}

impl SelectorArgs {
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
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// add a note, with --todo a task; the editor opens without -m
    Add {
        /// custom id, default: random
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

        /// a task (status todo) instead of a note
        #[arg(long)]
        todo: bool,

        /// due date, e.g. 2026-10-30
        #[arg(long, value_name = "DATE")]
        due: Option<String>,

        /// priority, 1 (high) .. 3 (low)
        #[arg(long, value_name = "N")]
        prio: Option<u64>,

        /// items that must be closed first
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,
    },
    /// edit the item file in the editor, or change body/priority/due directly
    Edit {
        id: String,

        /// replace the body, - reads stdin; frontmatter stays
        #[arg(short, long)]
        message: Option<String>,

        /// priority 1 (high) .. 3 (low), 0 clears
        #[arg(long, value_name = "N")]
        prio: Option<u64>,

        /// due date, e.g. 2026-10-30, - clears
        #[arg(long, value_name = "DATE")]
        due: Option<String>,
    },
    /// show an item, or any markdown file
    Show {
        /// item id or file path
        target: String,
    },
    /// list items
    #[command(visible_alias("ls"))]
    List {
        #[command(flatten)]
        selector: SelectorArgs,

        /// all fields and the body, also in json
        #[arg(short, long)]
        long: bool,
    },
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
    /// stop the timer, default: the running items
    Stop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set status done, stops the timer
    Done {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set status dropped, stops the timer
    Drop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set status todo: turns a note into a task or reopens it
    #[command(visible_alias("reopen"))]
    Todo {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// add tags, ~tag removes: cake tag [IDS] TAGS
    Tag {
        #[command(flatten)]
        selector: SelectorArgs,

        /// tags to add, ~tag to remove
        new_tags: Option<String>,
    },
    /// move an item under a parent
    Mv {
        id: String,
        /// parent id, - detaches
        parent: String,
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
    /// create a store; in a git repo on its own branch, attached if it exists on the remote
    Init {},
    /// open the config file in the editor
    Config {},
}

impl Commands {
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
                | Commands::Block { .. }
                | Commands::Unblock { .. }
                | Commands::Done { .. }
                | Commands::Drop { .. }
                | Commands::Todo { .. }
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

fn print_report(store: &Store, report: commands::Report) {
    let json = match report {
        commands::Report::None => serde_json::json!({ "v": 1 }),
        commands::Report::Items(ids) => serde_json::json!({
            "v": 1,
            "items": ids.iter().filter_map(|id| store.item_json(id)).collect::<Vec<_>>(),
        }),
        commands::Report::Brief(ids) => serde_json::json!({
            "v": 1,
            "items": ids
                .iter()
                .filter_map(|id| store.item_json(id))
                .map(|mut i| {
                    i.as_object_mut().unwrap().remove("body");
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

/// -m value, "-" reads stdin
fn read_message(message: &Option<String>) -> Result<Option<String>, Box<dyn Error>> {
    Ok(match message.as_deref() {
        Some("-") => Some(std::io::read_to_string(std::io::stdin())?),
        m => m.map(String::from),
    })
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

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) if args().any(|a| a == "--json") && e.use_stderr() => {
            out::set_json(true);
            return Err(error::invalid(e.to_string().trim()));
        }
        Err(e) => e.exit(),
    };
    out::set_json(cli.json);

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let debug = |s: &str| match cli.debug {
        0 => (),
        _ => eprintln!("{}", s.yellow()),
    };
    debug(&format!("debug mode is on."));

    let args: Vec<String> = args().collect();
    debug(&format!("{:?}", args));

    let config = Config::new()?;
    let root = match &cli.store {
        Some(p) => p.to_owned(),
        None if cli.global => util::global_store()?,
        None => util::find_store(&current_dir()?)?,
    };
    debug(&format!("Store: {}", root.display()));

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

    let report = match &cli.command {
        Some(Commands::Add {
            ids,
            children,
            parents,
            message,
            tags,
            todo,
            due,
            prio,
            blocked_by,
        }) => commands::add(
            &debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, parents, tags, &None, &None, &false, &false, &0, &false,
            )?,
            read_message(message)?,
            commands::AddOptions {
                todo: *todo,
                due: due.to_owned(),
                priority: *prio,
                blocked_by: util::split_comma_cleanup(blocked_by.to_owned().unwrap_or_default()),
            },
        )?,
        Some(Commands::Edit {
            id,
            message,
            prio,
            due,
        }) => commands::edit(
            &debug,
            &config,
            &mut store,
            id,
            commands::EditOptions {
                body: read_message(message)?,
                priority: *prio,
                due: due.to_owned(),
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
            commands::start(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Stop { selector }) => {
            commands::stop(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Mv { id, parent }) => commands::mv(debug, &mut store, id, parent)?,
        Some(Commands::Block { id, by }) => commands::block(debug, &mut store, id, by, false)?,
        Some(Commands::Unblock { id, by }) => commands::block(debug, &mut store, id, by, true)?,
        Some(Commands::Ready { selector, long }) => {
            let mut selector = selector.selector(&selector.ids)?;
            selector.ready = true;
            commands::list(debug, &config, &mut store, selector, *long)?
        }
        Some(Commands::Done { selector }) => {
            commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Done)?
        }
        Some(Commands::Drop { selector }) => commands::set_status(
            debug,
            &mut store,
            selector.selector(&selector.ids)?,
            Status::Dropped,
        )?,
        Some(Commands::Todo { selector }) => {
            commands::set_status(debug, &mut store, selector.selector(&selector.ids)?, Status::Todo)?
        }
        Some(Commands::List { selector, long }) => commands::list(
            debug,
            &config,
            &mut store,
            selector.selector(&selector.ids)?,
            *long,
        )?,
        Some(Commands::Show { target }) => commands::show(debug, &config, &store, target)?,
        Some(Commands::Init {}) => commands::init(debug, &config, cli.global)?,
        Some(Commands::Config {}) => commands::config(debug, &config)?,
        Some(Commands::Sync { .. }) => unreachable!("handled before loading the store"),
        None => {
            if !out::json() {
                Cli::command().print_help()?;
            }
            commands::Report::None
        }
    };

    let changed = store.save()?;
    if !config.disable_git && git::is_store_repo(&root) {
        let message = commit_message(&args[1..], util::agent());
        debug(&format!("git commit {:?}: {}", changed, message));
        git::commit(&root, &changed, &message)?;
    }
    config.write_json_if_dirty()?;
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
    fn tag_parses_ids_before_new_tags_and_flattened_selector_flags() {
        let cli =
            Cli::try_parse_from(["cake", "tag", "a,b", "x,y", "-t", "~z", "-rr", "--or"]).unwrap();
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
