mod commands;
mod item;
mod store;
mod util;

use crate::store::Store;
use clap::{Parser, Subcommand};
use colored::*;
use commands::Selector;
use std::{
    env::{args, current_dir},
    error::Error,
    path::PathBuf,
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
    },
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
                ids, children, parents, tags, &None, &None, &false, &false, &0
            ),
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
            recursive
        }) => commands::remove(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive
            ),
        )?,
        Some(Commands::Start {
            ids,
            children,
            parents,
            tags,
            before,
            after,
            started,
            stopped,
            recursive
        }) => commands::start(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive
            ),
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
            recursive
        }) => commands::stop(
            debug,
            &mut &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive
            ),
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
        }) => commands::list(
            debug,
            &mut store,
            Selector::new(
                ids, children, parents, tags, before, after, started, stopped, recursive
            ),
            *long,
        )?,
        None => {
            println!("Nothing happed o.0");
        }
    }

    store.write(&file)?;
    Ok(())
}
