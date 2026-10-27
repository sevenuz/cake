mod commands;
mod config;
mod error;
mod git;
mod item;
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

    /// print debug output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

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

    /// task|note
    #[arg(long)]
    kind: Option<String>,

    /// todo|doing|done|dropped
    #[arg(long)]
    status: Option<String>,

    /// open, no open blockers
    #[arg(long)]
    ready: bool,

    /// open, with open blockers
    #[arg(long)]
    blocked: bool,
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
        .with_filters(&self.search, &self.kind, &self.status, self.ready, self.blocked)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// add an item; the editor opens without -m or --body-file
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

        /// content; the first `# heading` is the title
        #[arg(short, long, conflicts_with = "body_file")]
        message: Option<String>,

        /// read content from a file, - for stdin
        #[arg(long, value_name = "FILE")]
        body_file: Option<PathBuf>,

        /// a note instead of a task
        #[arg(long)]
        note: bool,

        /// due date, e.g. 2026-10-30
        #[arg(long, value_name = "DATE")]
        due: Option<String>,

        /// priority, 1 (high) .. 3 (low)
        #[arg(long, value_name = "N")]
        prio: Option<u64>,

        /// items that must be closed first
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,

        /// related items
        #[arg(long, value_name = "IDS")]
        related: Option<String>,
    },
    /// edit the item file in the editor
    Edit { id: String },
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

        /// all fields and the body
        #[arg(short, long)]
        long: bool,
    },
    /// list open items without open blockers, by priority and due
    Ready {
        #[command(flatten)]
        selector: SelectorArgs,

        /// all fields and the body
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
    /// close tasks as done
    Done {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// close tasks as dropped
    Drop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set tasks back to todo
    Reopen {
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
                | Commands::Reopen { .. }
        )
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let debug = |s: &str| match cli.debug {
        0 => (),
        _ => println!("{}", s.yellow()),
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
        return commands::sync(debug, &config, &root, publish, *cont, *abort);
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
            return Err(msg.into());
        }
        eprintln!("{}", msg.yellow());
    }
    let skip: Vec<PathBuf> = conflicted.iter().map(|f| root.join(f)).collect();
    let mut store = Store::new_except(&root, &skip)?;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Add {
            ids,
            children,
            parents,
            message,
            tags,
            body_file,
            note,
            due,
            prio,
            blocked_by,
            related,
        }) => commands::add(
            &debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, parents, tags, &None, &None, &false, &false, &0, &false,
            )?,
            match body_file {
                Some(f) if f.as_os_str() == "-" => Some(std::io::read_to_string(std::io::stdin())?),
                Some(f) => Some(std::fs::read_to_string(f)?),
                None => message.to_owned(),
            },
            commands::AddOptions {
                note: *note,
                due: due.to_owned(),
                priority: *prio,
                blocked_by: util::split_comma_cleanup(blocked_by.to_owned().unwrap_or_default()),
                related: util::split_comma_cleanup(related.to_owned().unwrap_or_default()),
            },
        )?,
        Some(Commands::Edit { id }) => commands::edit(&debug, &config, &mut store, id)?,
        Some(Commands::Remove { selector }) => {
            commands::remove(debug, &mut &mut store, selector.selector(&selector.ids)?)?
        }
        Some(Commands::Tag { selector, new_tags }) => {
            let mut i = &selector.ids;
            let mut nt = new_tags;
            // use ids for tags if only one option is set.
            if i.is_none() && nt.is_none() {
                return Err("You have to specify tags. [ids selector, optional] [tags]".into());
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
        Some(Commands::Reopen { selector }) => {
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
        None => Cli::command().print_help()?,
    }

    let changed = store.save()?;
    if !config.disable_git && git::is_store_repo(&root) {
        let message = format!("cake {}", args[1..].join(" "));
        debug(&format!("git commit {:?}: {}", changed, message));
        git::commit(&root, &changed, &message)?;
    }
    config.write_json_if_dirty()?;
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
