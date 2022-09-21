mod store;
mod item;
mod util;

use clap::{Parser, Subcommand};
use std::error::Error;
use crate::item::{Item, generate_id};
use crate::util::*;
use crate::store::Store;

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
        /// the todo item, if non, editor is opened
        #[clap(value_parser)]
        message: Option<String>,

        /// the children items wich the new one is linked to
        #[clap(short, long)]
        children: Option<String>,

        /// the parent items the new one is linked to
        #[clap(short, long)]
        parents: Option<String>,

        /// Optional tags seperated by colon
        #[clap(short, long)]
        tags: Option<String>,

        /// a custom unique id for the item
        #[clap(short, long)]
        id: Option<String>,

        /// updates the item with the provided id if found
        #[clap(short, long, action)]
        edit: bool,
    },
    /// remove items
    #[clap(alias("rm"))]
    Remove {
        /// the item which should be removed
        #[clap(value_parser)]
        id: String,

        /// recursive removing all linged items
        #[clap(short, long, action)]
        recursive: bool,
    },
    /// list items
    #[clap(alias("ls"))]
    List {
        /// the item which should be listed
        #[clap(value_parser)]
        id: Option<String>,

        /// recursive removing all linked items
        #[clap(short, long, action)]
        recursive: bool,

        /// detailed presentation of the items
        #[clap(short, long, action)]
        long: bool,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let file = cli.config.unwrap_or("./cake.json".to_string());
    let mut store = Store::new(&file);

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let _debug: bool = cli.debug > 0;
    match cli.debug {
        0 => print!(""),
        1 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Add { message, children, parents, id, tags, edit }) => {
            let _id = &id.to_owned().unwrap_or("".to_string());
            if *edit {
                store.edit(_id, message, children, parents, tags).unwrap_or_else(|err| {
                    println!("{}", err);
                });
            } else {
                let item = Item::new(
                    remove_comma(if _id.is_empty() { generate_id() } else { _id.to_string() }),
                    split_comma(children.to_owned().unwrap_or("".to_string())),
                    split_comma(parents.to_owned().unwrap_or("".to_string())),
                    split_comma(tags.to_owned().unwrap_or("".to_string())),
                    message.to_owned().unwrap_or("".to_string())
                );
                store.add(item).unwrap_or_else(|err| {
                    println!("{}", err);
                });
            }
            println!("{}", store.get_mut_item(_id).unwrap());
            store.write(&file);
        }
        Some(Commands::Remove { id, recursive }) => {
            if _debug {
                println!("id: {:?} recursive: {:?}", id, recursive);
            }
            store.remove(id, *recursive);
            store.write(&file);
            println!("{:?} removed.", id); // TODO print count of deleted items
        }
        Some(Commands::List { id, recursive, long }) => {
            if _debug {
                println!("id: {:?} recursive: {:?} long: {:?}", id, recursive, long);
            }
            let _id = &id.to_owned().unwrap_or("".to_string());
            let _items = store.get();
            let mut _cycle: Vec<String> = vec![];
            let max_depth = if *recursive { 10 /*std::usize::MAX*/ } else { 1 };

            fn prl(item: &Item, depth: usize) {
                println!("{:indent$}{}", "", item, indent=depth);
            }
            fn prs(item: &Item, depth: usize) {
                println!("{:indent$}{}", "", item.print(), indent=depth);
            }

            if _id.is_empty() {
                let mut _keys = _items.keys().cloned().collect::<Vec<String>>();
                // sort output from old to new
                _keys.sort_by(|a, b| {
                    _items.get(a).unwrap().timestamp.cmp(&_items.get(b).unwrap().timestamp)
                });
                // sort output by amount of parents. Zero parents first
                _keys.sort_by(|a, b| {
                    _items.get(a).unwrap().parents.len().cmp(&_items.get(b).unwrap().parents.len())
                });
                store.recursive_execute(&_keys, &mut _cycle, if *long { prl } else { prs }, 0, max_depth);
            } else {
                let _keys = vec![_id.to_owned()];
                store.recursive_execute(&_keys, &mut _cycle, if *long { prl } else { prs }, 0, max_depth);
            }
        }
        None => {
            println!("Nothing happed o.0");
        }
    }

    Ok(())
}
