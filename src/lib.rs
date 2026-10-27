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
use clap::{Args, Parser, Subcommand};
use config::Config;
use item::Status;
use selector::Selector;
use std::{
    env::{args, current_dir},
    error::Error,
    path::PathBuf,
    process::exit,
};
use termimad::crossterm::style::Stylize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Use the store at this path instead of the nearest .cake or the global store
    #[arg(long, value_name = "DIR", env = "CAKE_STORE")]
    pub store: Option<PathBuf>,

    /// Use the global store, also inside a project
    #[arg(short, long, conflicts_with = "store")]
    pub global: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args)]
pub struct SelectorArgs {
    /// Select by ids
    ids: Option<String>,

    /// Select by children
    #[arg(short, long)]
    children: Option<String>,

    /// Select by parents
    #[arg(short, long)]
    parents: Option<String>,

    /// Select by tags, use ~ to exclude tag e.g. -t ~some_tag
    #[arg(short, long)]
    tags: Option<String>,

    /// Select by time before this duration from now
    #[arg(short, long)]
    before: Option<String>,

    /// Select by time after this duration from now
    #[arg(short, long)]
    after: Option<String>,

    /// Select started items
    #[arg(long)]
    started: bool,

    /// Select stopped items
    #[arg(long)]
    stopped: bool,

    /// recursive execution of the command. -r: children, -rr parents, -rrr both
    #[arg(short, long, action = clap::ArgAction::Count)]
    recursive: u8,

    /// Concatinate selectors with or instead of and
    #[arg(long)]
    or: bool,

    /// Select by text in title or body, case-insensitive
    #[arg(short, long, value_name = "TEXT")]
    search: Option<String>,

    /// Select by kind: task|note
    #[arg(long)]
    kind: Option<String>,

    /// Select by status: todo|doing|done|dropped, comma separated
    #[arg(long)]
    status: Option<String>,

    /// Select open items without open blockers
    #[arg(long)]
    ready: bool,

    /// Select open items with open blockers
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

// TODO https://docs.rs/clap/latest/clap/builder/struct.Arg.html#method.value_delimiter
#[derive(Subcommand)]
pub enum Commands {
    /// add a new item, opens the editor without -m
    Add {
        /// custom id, default: random
        ids: Option<String>,

        /// children to move under the new item, comma separated
        #[arg(short, long)]
        children: Option<String>,

        /// parent item
        #[arg(short, long)]
        parents: Option<String>,

        /// tags, comma separated
        #[arg(short, long)]
        tags: Option<String>,

        /// content, the first `# heading` is the title
        #[arg(short, long, conflicts_with = "body_file")]
        message: Option<String>,

        /// read the content from a file, - for stdin
        #[arg(long, value_name = "FILE")]
        body_file: Option<PathBuf>,

        /// add a note instead of a task
        #[arg(long)]
        note: bool,

        /// due date, e.g. 2026-10-30
        #[arg(long)]
        due: Option<String>,

        /// priority, 1 (high) .. 3 (low)
        #[arg(long)]
        prio: Option<u64>,

        /// items that must be closed first, comma separated
        #[arg(long, value_name = "IDS")]
        blocked_by: Option<String>,

        /// related items, comma separated
        #[arg(long, value_name = "IDS")]
        related: Option<String>,
    },
    /// open the item file (frontmatter and body) in the editor
    Edit {
        id: String,
    },
    /// remove items, alias rm
    #[command(alias("rm"))]
    Remove {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// Tag items with selectors.
    /// Use ~tag to remove a tag.
    Tag {
        #[command(flatten)]
        selector: SelectorArgs,

        /// comma separated tags which will be assigned
        new_tags: Option<String>,
    },
    /// start timetracking for item
    Start {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// stop timetracking for item, without selector the running ones
    Stop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// move an item under a new parent, - detaches it
    Mv {
        id: String,
        /// new parent id or -
        parent: String,
    },
    /// mark tasks done: stops the timer, sets closed
    Done {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// mark tasks dropped: stops the timer, sets closed
    Drop {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// set tasks back to todo
    Reopen {
        #[command(flatten)]
        selector: SelectorArgs,
    },
    /// list items, alias ls
    #[command(alias("ls"))]
    List {
        #[command(flatten)]
        selector: SelectorArgs,

        /// detailed presentation of the items
        #[arg(short, long)]
        long: bool,
    },
    /// show a markdown file in terminal
    Show {
        /// Path to the file
        path: String,
    },
    /// Creates a new store (.cake). In a git repo it is a worktree of an orphan
    /// branch (config git_branch), attached if the branch exists on the remote.
    Init {},
    /// Fetch, merge and push the store. Only runs when invoked.
    Sync {
        /// Push the store for the first time and set the upstream (default remote: config git_remote_name)
        #[arg(long, value_name = "REMOTE", num_args = 0..=1, default_missing_value = "", group = "mode")]
        publish: Option<String>,

        /// Commit the resolved merge and push
        #[arg(long = "continue", group = "mode")]
        cont: bool,

        /// Abort the merge in progress
        #[arg(long, group = "mode")]
        abort: bool,
    },
    /// Open config file in editor
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
                | Commands::Done { .. }
                | Commands::Drop { .. }
                | Commands::Reopen { .. }
        )
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::try_parse();

    if let Err(err) = cli {
        // TODO search for commands in settings
        println!("Sheesh, a {}", err);
        exit(1);
    }

    let cli = cli.unwrap();

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
        Some(Commands::Show { path }) => commands::show(debug, &config, path)?,
        Some(Commands::Init {}) => commands::init(debug, &config, cli.global)?,
        Some(Commands::Config {}) => commands::config(debug, &config)?,
        Some(Commands::Sync { .. }) => unreachable!("handled before loading the store"),
        None => {
            println!("Nothing happed o.0");
        }
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
