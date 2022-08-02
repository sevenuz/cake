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
    /// add items
    Add {
        /// the todo item, if non, editor is opened
        #[clap(value_parser)]
        message: Option<String>,

        /// the item the new one is linked to
        #[clap(value_parser)]
        linked_items: Option<String>,

        /// Optional tags seperated by colon
        #[clap(short, long)]
        tags: Option<String>,

        /// a custom unique id for the item
        #[clap(short, long)]
        id: Option<String>,
    },
    /// remove items
    Remove {
        /// the item which should be removed
        #[clap(value_parser)]
        id: String,

        /// recursive removing all linged items
        #[clap(short, long, action)]
        recursive: bool,
    },
    /// list items
    List {
        /// the item which should be listed
        #[clap(value_parser)]
        id: Option<String>,

        /// recursive removing all linked items
        #[clap(short, long, action)]
        recursive: bool,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => print!(""),
        1 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Add { message, linked_items, id, tags }) => {
            let item = Item::new(
                remove_comma(id.to_owned().unwrap_or(generate_id())),
                split_comma(linked_items.to_owned().unwrap_or("".to_string())),
                split_comma(tags.to_owned().unwrap_or("".to_string())),
                message.to_owned().unwrap_or("".to_string())
            );
            println!("Added new Todo with following Id: {:?}", item.id);
            println!("\"{}\"", item.content);
            let file = cli.config.unwrap_or("./cake.json".to_string());
            let mut store = Store::new(&file);
            store.add(item);
            store.write(&file);
        }
        Some(Commands::Remove { id, recursive }) => {
            println!("id: {:?} recursive: {:?}", id, recursive);
        }
        Some(Commands::List { id, recursive }) => {
            println!("id: {:?} recursive: {:?}", id, recursive);
        }
        None => {
            println!("Nothing happed o.0");
        }
    }

    Ok(())
}

