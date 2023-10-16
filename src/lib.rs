mod commands;
mod item;
mod store;
mod util;
mod selector;

use crate::store::Store;
use clap::{Parser, Subcommand};
use colored::*;
use selector::Selector;
use std::{
    env::{args, current_dir},
    error::Error,
    path::PathBuf,
    process::exit,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub config: Option<String>,

    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

// TODO https://docs.rs/clap/latest/clap/builder/struct.Arg.html#method.value_delimiter
#[derive(Subcommand)]
pub enum Commands {
    /// add or edit items
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
    /// remove items
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
    /// tag items with selectors
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
    /// list items
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

    // use empty config path to write to global save file
    let file = match cli.config {
        Some(f) => {
            if f.is_empty() {
                util::find_save_file(&mut PathBuf::new()).unwrap()
            } else {
                f
            }
        }
        None => util::find_save_file(&mut current_dir()?).unwrap(),
    };
    debug(&format!("File: {}", file));

    let mut store = Store::new(&file)?;

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
                nt
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
            &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive, or,
            )?,
            *long,
        )?,
        None => {
            println!("Nothing happed o.0");
        }
    }

    store.write(&file)?;
    Ok(())
}
