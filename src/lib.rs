mod item;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use std::error::Error;
use crate::item::Item;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub config: Option<PathBuf>,

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

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

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
            println!("todo: {:?} linked_items {:?} id: {:?} tags: {:?}", message, linked_items, id, tags);
            let item = Item{id: String::from("123"), linked_items: vec!["-w".to_string()], tags: vec!["-w".to_string()], content: String::from("...")};
            let serialized = serde_json::to_string_pretty(&item).unwrap();
            let _ditem: Item = serde_json::from_str(&serialized).unwrap();
            println!("todo: {:?} linked_items {:?} id: {:?} tags: {:?}", _ditem.content, _ditem.linked_items, _ditem.id, _ditem.tags);
            std::fs::write("./test.json", serialized).unwrap();
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

