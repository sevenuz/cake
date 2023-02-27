mod commands;
mod item;
mod store;
mod util;

use crate::item::Item;
use crate::store::inner::RecState;
use crate::store::Store;
use clap::{Parser, Subcommand};
use colored::*;
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
        id: Option<String>,

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
        /// the item which should be removed
        #[clap(value_parser)]
        id: String,

        /// recursive removing all linked items
        #[clap(short, long, action)]
        recursive: bool,
    },
    /// start timetracking for item
    Start {
        /// the item which should be removed
        #[clap(value_parser)]
        id: String,
    },
    /// stop timetracking for item
    Stop {
        /// the item which should be removed
        #[clap(value_parser)]
        id: String,
    },
    /// list items
    #[clap(alias("ls"))]
    List {
        /// the item which should be listed
        #[clap(value_parser)]
        id: Option<String>,

        /// recursive show all linked items
        #[clap(short, long, action)]
        recursive: bool,

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
            id,
            children,
            parents,
            message,
            tags,
            edit,
            overwrite,
        }) => {
            // auto edit flag only works if no flags are used before...
            let _edit = args[1] == "edit" || *edit;
            commands::add(
                &debug,
                &mut store,
                util::split_comma(util::remove_illegal_characters(
                    id.to_owned().unwrap_or("".to_string()),
                )),
                util::split_comma(util::remove_illegal_characters(
                    children.to_owned().unwrap_or("".to_string()),
                )),
                util::split_comma(util::remove_illegal_characters(
                    parents.to_owned().unwrap_or("".to_string()),
                )),
                util::split_comma(util::remove_illegal_characters(
                    tags.to_owned().unwrap_or("".to_string()),
                )),
                message.to_owned().unwrap_or("".to_string()),
                _edit,
                *overwrite,
            )?;
        }
        Some(Commands::Remove { id, recursive }) => {
            debug(&format!("id: {:?} recursive: {:?}", id, recursive));
            store.remove(id, *recursive)?;
            println!("{:?} removed.", id); // TODO print count of deleted items
        }
        Some(Commands::Start { id }) => {
            match store.get_item_mut(id).expect("Could not found id").start() {
                Ok(_) => {
                    println!("{:?} started.", id); // TODO print count of deleted items
                    store.write(&file)?;
                }
                Err(err) => return Err(err.into()),
            }
        }
        Some(Commands::Stop { id }) => {
            match store.get_item_mut(id).expect("Could not found id").stop() {
                Ok(_) => {
                    println!("{:?} stoped.", id); // TODO print count of deleted items
                }
                Err(err) => return Err(err.into()),
            }
        }
        Some(Commands::List {
            id,
            recursive,
            long,
        }) => {
            debug(&format!(
                "id: {:?} recursive: {:?} long: {:?}",
                id, recursive, long
            ));
            let _id = &id.to_owned().unwrap_or("".to_string());
            let _items = store.get();
            let mut _cycle: Vec<String> = vec![];
            let max_depth = if *recursive {
                10 /*std::usize::MAX*/
            } else {
                1
            };

            fn print_line(line: String, depth: usize, state: &RecState) {
                match state {
                    RecState::Normal => println!("{:indent$}{}", "", line, indent = depth),
                    RecState::Reappearence if depth > 0 => {
                        println!("{:indent$}{}", "", line.bright_green(), indent = depth)
                    }
                    _ => (),
                }
            }

            fn prl(item: &Item, depth: usize, state: RecState) {
                // double indention for visuality reasons
                let indent = 10 * depth;
                for line in item.to_string().split("\n") {
                    print_line(line.to_string(), indent, &state);
                }
                if !(matches!(state, RecState::Reappearence) && depth == 0) {
                    println!(
                        "{}",
                        "=====================================================".red()
                    );
                }
            }

            fn prs(item: &Item, depth: usize, state: RecState) {
                print_line(item.print(), depth, &state);
            }

            if _id.is_empty() {
                let mut _keys = _items.keys().cloned().collect::<Vec<String>>();
                // sort output from old to new
                _keys.sort_by(|a, b| {
                    _items
                        .get(a)
                        .unwrap()
                        .timestamp
                        .cmp(&_items.get(b).unwrap().timestamp)
                });
                // sort output by amount of parents. Zero parents first
                _keys.sort_by(|a, b| {
                    _items
                        .get(a)
                        .unwrap()
                        .parents
                        .len()
                        .cmp(&_items.get(b).unwrap().parents.len())
                });
                store.recursive_execute(
                    &_keys,
                    &mut _cycle,
                    if *long { prl } else { prs },
                    0,
                    max_depth,
                );
            } else {
                store.check_id(_id, true)?;
                let _keys = vec![_id.to_owned()];
                store.recursive_execute(
                    &_keys,
                    &mut _cycle,
                    if *long { prl } else { prs },
                    0,
                    max_depth,
                );
            }
        }
        None => {
            println!("Nothing happed o.0");
        }
    }

    store.write(&file)?;
    Ok(())
}
