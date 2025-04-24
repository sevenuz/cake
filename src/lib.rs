mod commands;
mod config;
mod error;
mod git;
mod item;
mod selector;
mod store;
mod util;
mod view;

use crate::store::Store;
use clap::{Parser, Subcommand};
use config::Config;
use selector::Selector;
use std::{env::args, error::Error, process::exit};
use termimad::crossterm::style::Stylize;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom input db file.
    /// Use an empty input path to write to the global save file.
    /// The format is determined by the file extension: json or md
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub input: Option<String>,

    /// Sets a custom output db file. If not set, the input file is used.
    /// The format is determined by the file extension: json or md
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub output: Option<String>,

    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

// TODO https://docs.rs/clap/latest/clap/builder/struct.Arg.html#method.value_delimiter
#[derive(Subcommand)]
pub enum Commands {
    /// add new items, alias edit: sets --edit
    #[clap(alias("edit"))]
    Add {
        /// a custom unique id for the item
        #[clap(value_parser)]
        ids: Option<String>,

        /// the children items wich the new one is linked to
        #[clap(short, long)]
        children: Option<String>,

        /// the parent items the new one is linked to
        #[clap(short, long)]
        parents: Option<String>,

        /// Optional tags seperated by colon
        #[clap(short, long)]
        tags: Option<String>,

        /// the todo content, if non, editor is opened
        #[clap(short, long)]
        message: Option<String>,

        /// updates the item with the provided id if found
        #[clap(short, long, action)]
        edit: bool,

        /// overwrites the item with the provided id if found, edit flag is required
        #[clap(short, long, action)]
        overwrite: bool,
    },
    /// remove items, alias rm
    #[clap(alias("rm"))]
    Remove {
        /// Select by ids
        #[clap(value_parser)]
        ids: Option<String>,

        /// Select by children
        #[clap(short, long)]
        children: Option<String>,

        /// Select by parents
        #[clap(short, long)]
        parents: Option<String>,

        /// Select by tags
        /// use ~ to exclude tag e.g. -t ~some_tag
        #[clap(short, long)]
        tags: Option<String>,

        /// Select by time before this duration from now
        #[clap(short, long)]
        before: Option<String>,

        /// Select by time after this duration from now
        #[clap(short, long)]
        after: Option<String>,

        /// Select started items
        #[clap(long, action)]
        started: bool,

        /// Select stopped items
        #[clap(long, action)]
        stopped: bool,

        /// recursive execution of the command.
        /// e.g. -r: children, -rr parents, -rrr both
        #[clap(short, long, action = clap::ArgAction::Count)]
        recursive: u8,

        /// Concatinate selectors with or instead of and
        #[clap(long, action)]
        or: bool,
    },
    /// Tag items with selectors.
    /// Use ~tag to remove a tag.
    Tag {
        /// Select by ids
        #[clap(value_parser)]
        ids: Option<String>,

        /// comma separated tags which will be assigned
        #[clap(value_parser)]
        new_tags: Option<String>,

        /// Select by children
        #[clap(short, long)]
        children: Option<String>,

        /// Select by parents
        #[clap(short, long)]
        parents: Option<String>,

        /// Select by tags
        /// use ~ to exclude tag e.g. -t ~some_tag
        #[clap(short, long)]
        tags: Option<String>,

        /// Select by time before this duration from now
        #[clap(short, long)]
        before: Option<String>,

        /// Select by time after this duration from now
        #[clap(short, long)]
        after: Option<String>,

        /// Select started items
        #[clap(long, action)]
        started: bool,

        /// Select stopped items
        #[clap(long, action)]
        stopped: bool,

        /// recursive execution of the command.
        /// e.g. -r: children, -rr parents, -rrr both
        #[clap(short, long, action = clap::ArgAction::Count)]
        recursive: u8,

        /// Concatinate selectors with or instead of and
        #[clap(long, action)]
        or: bool,
    },
    /// start timetracking for item
    Start {
        /// Select by ids
        #[clap(value_parser)]
        ids: Option<String>,

        /// Select by children
        #[clap(short, long)]
        children: Option<String>,

        /// Select by parents
        #[clap(short, long)]
        parents: Option<String>,

        /// Select by tags, use ~ to exclude tag e.g. -t ~some_tag
        #[clap(short, long)]
        tags: Option<String>,

        /// Select by time before this duration from now
        #[clap(short, long)]
        before: Option<String>,

        /// Select by time after this duration from now
        #[clap(short, long)]
        after: Option<String>,

        /// Select started items
        #[clap(long, action)]
        started: bool,

        /// Select stopped items
        #[clap(long, action)]
        stopped: bool,

        /// recursive execution of the command. -r: children, -rr parents, -rrr both
        #[clap(short, long, action = clap::ArgAction::Count)]
        recursive: u8,

        /// Concatinate selectors with or instead of and
        #[clap(long, action)]
        or: bool,
    },
    /// stop timetracking for item
    Stop {
        /// Select by ids
        #[clap(value_parser)]
        ids: Option<String>,

        /// Select by children
        #[clap(short, long)]
        children: Option<String>,

        /// Select by parents
        #[clap(short, long)]
        parents: Option<String>,

        /// Select by tags, use ~ to exclude tag e.g. -t ~some_tag
        #[clap(short, long)]
        tags: Option<String>,

        /// Select by time before this duration from now
        #[clap(short, long)]
        before: Option<String>,

        /// Select by time after this duration from now
        #[clap(short, long)]
        after: Option<String>,

        /// Select started items
        #[clap(long, action)]
        started: bool,

        /// Select stopped items
        #[clap(long, action)]
        stopped: bool,

        /// recursive execution of the command. -r: children, -rr parents, -rrr both
        #[clap(short, long, action = clap::ArgAction::Count)]
        recursive: u8,

        /// Concatinate selectors with or instead of and
        #[clap(long, action)]
        or: bool,
    },
    /// list items, alias ls
    #[clap(alias("ls"))]
    List {
        /// Select by ids
        #[clap(value_parser)]
        ids: Option<String>,

        /// Select by children
        #[clap(short, long)]
        children: Option<String>,

        /// Select by parents
        #[clap(short, long)]
        parents: Option<String>,

        /// Select by tags, use ~ to exclude tag e.g. -t ~some_tag
        #[clap(short, long)]
        tags: Option<String>,

        /// Select by time before this duration from now
        #[clap(short, long)]
        before: Option<String>,

        /// Select by time after this duration from now
        #[clap(short, long)]
        after: Option<String>,

        /// Select started items
        #[clap(long, action)]
        started: bool,

        /// Select stopped items
        #[clap(long, action)]
        stopped: bool,

        /// recursive execution of the command. -r: children, -rr parents, -rrr both
        #[clap(short, long, action = clap::ArgAction::Count)]
        recursive: u8,

        /// detailed presentation of the items
        #[clap(short, long, action)]
        long: bool,

        /// Concatinate selectors with or instead of and
        #[clap(long, action)]
        or: bool,
    },
    /// show a markdown file in terminal
    Show {
        /// Path to the file
        #[clap(value_parser)]
        path: String,
    },
    /// Creates new cake save file in current directory, checkout `cake help init` for git options.
    Init {
        /// Creates new branch for the cake save file which is checked out
        /// before every cake operation.
        #[clap(long, action)]
        git: bool,
        /// Wether the cake branch should be pushed to remote or not.
        /// This option works only together with the git option.
        #[clap(long, action)]
        remote: bool,
    },
}

const FILETYPE_JSON: &str = ".json";
const FILETYPE_MD: &str = ".md";

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
    let mut current_branch = "".to_string();
    if git::is_repo() {
        current_branch = git::current_branch_name()?;
        if git::check_if_branch_exists(&config)? {
            debug(&format!(
                "detected git repo and cake branch. current branch: {:?}",
                current_branch
            ));
            debug(&format!("git stash"));
            git::stash()?;
            debug(&format!("git checkout {:?}", config.git_branch_name));
            git::checkout_branch(&config.git_branch_name)?;
        }
    }
    let input_file = match cli.input {
        Some(f) => {
            // use empty input path to write to global save file
            if f.is_empty() {
                config.get_default_file_path()
            } else {
                f
            }
        }
        None => config.find_save_file()?,
    };
    let output_file = match cli.output {
        Some(f) if !f.is_empty() => f,
        None | _ => input_file.to_owned(),
    };
    debug(&format!("Input: {}, Output: {}", input_file, output_file));

    let mut store: Store;
    if input_file.ends_with(FILETYPE_MD) {
        store = Store::new_from_md(&input_file)?;
    } else if input_file.ends_with(FILETYPE_JSON) {
        store = Store::new_from_json(&input_file)?;
    } else {
        return Err("Only .md or .json files are supported".into());
    }

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
        Some(Commands::Remove {
            ids,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive,
            or,
        }) => commands::remove(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive, or,
            )?,
        )?,
        Some(Commands::Tag {
            ids,
            new_tags,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive,
            or,
        }) => {
            let mut i = ids;
            let mut nt = new_tags;
            // use ids for tags if only one option is set.
            if ids.is_none() && new_tags.is_none() {
                return Err("You have to specify tags. [ids selector, optional] [tags]".into());
            } else if new_tags.is_none() {
                nt = ids;
                i = new_tags;
            }
            commands::tag(
                debug,
                &mut &mut store,
                Selector::new(
                    i, children, parents, tags, before, after, started, stopped, recursive, or,
                )?,
                nt,
            )?
        }
        Some(Commands::Start {
            ids,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive,
            or,
        }) => commands::start(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive, or,
            )?,
        )?,
        Some(Commands::Stop {
            ids,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive,
            or,
        }) => commands::stop(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive, or,
            )?,
        )?,
        Some(Commands::List {
            ids,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive,
            long,
            or,
        }) => commands::list(
            debug,
            &config,
            &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive, or,
            )?,
            *long,
        )?,
        Some(Commands::Show { path }) => commands::show(debug, &config, path)?,
        Some(Commands::Init { git, remote }) => commands::init(debug, &config, *git, *remote)?,
        None => {
            println!("Nothing happed o.0");
        }
    }

    if output_file.ends_with(FILETYPE_MD) {
        store.write_md(&output_file)?;
    } else if output_file.ends_with(FILETYPE_JSON) {
        store.write_json(&output_file)?;
    } else {
        // TODO support all files if extension is set in settings
        return Err("Only .md or .json files are supported".into());
    }
    if git::is_repo() {
        if git::check_if_branch_exists(&config)? {
            debug(&format!(
                "detected git repo and cake branch. current branch: {:?}",
                current_branch
            ));
            debug(&format!("git add {:?}", config.save_file_name));
            git::add(&config)?;
            debug(&format!("git commit"));
            git::commit("cake: TODO better message")?;
            debug(&format!("git checkout {:?}", current_branch));
            git::checkout_branch(&current_branch)?;
            debug(&format!("git stash pop"));
            git::stash_pop()?;
        }
    }
    config.write_json_if_dirty()?;
    Ok(())
}
