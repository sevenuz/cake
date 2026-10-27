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
        )
    }
}

// TODO https://docs.rs/clap/latest/clap/builder/struct.Arg.html#method.value_delimiter
#[derive(Subcommand)]
pub enum Commands {
    /// add new items, alias edit: sets --edit
    #[command(alias("edit"))]
    Add {
        /// a custom unique id for the item
        ids: Option<String>,

        /// the children items wich the new one is linked to
        #[arg(short, long)]
        children: Option<String>,

        /// the parent items the new one is linked to
        #[arg(short, long)]
        parents: Option<String>,

        /// Optional tags seperated by colon
        #[arg(short, long)]
        tags: Option<String>,

        /// the todo content, if non, editor is opened
        #[arg(short, long)]
        message: Option<String>,

        /// updates the item with the provided id if found
        #[arg(short, long)]
        edit: bool,

        /// overwrites the item with the provided id if found, edit flag is required
        #[arg(short, long)]
        overwrite: bool,
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
    /// stop timetracking for item
    Stop {
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
    /// Open config file in editor
    Config {},
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
        None => util::find_store(&current_dir()?)?,
    };
    debug(&format!("Store: {}", root.display()));
    let mut store = Store::new(&root)?;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Add {
            ids,
            children,
            parents,
            message,
            tags,
            edit,
            overwrite,
        }) => commands::add(
            &debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, parents, tags, &None, &None, &false, &false, &0, &false,
            )?,
            message.to_owned().unwrap_or("".to_string()),
            args[1] == "edit" || *edit, // auto edit flag only works if no flags are used before...
            *overwrite,
        )?,
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
        Some(Commands::List { selector, long }) => commands::list(
            debug,
            &config,
            &mut store,
            selector.selector(&selector.ids)?,
            *long,
        )?,
        Some(Commands::Show { path }) => commands::show(debug, &config, path)?,
        Some(Commands::Init {}) => commands::init(debug, &config)?,
        Some(Commands::Config {}) => commands::config(debug, &config)?,
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
