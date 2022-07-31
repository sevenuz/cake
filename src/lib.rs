mod item;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use std::error::Error;
use crate::item::{Item, generate_id, write_items, read_items};

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

fn split_comma(s: String) -> Vec<String> {
    // return empty vector if input is ""
    if s == "" {
        return vec![];
    }
    // TODO improvement!!!
    let ca: Vec<&str> = s.split(",").collect();
    let mut vec: Vec<String> = Vec::new();
    ca.into_iter().for_each(|ll| {
        vec.push(ll.to_string());
    });
    return vec;
}

fn remove_comma(s: String) -> String {
    s.replace(",", "")
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
            let item = Item{
                id: remove_comma(id.to_owned().unwrap_or(generate_id())),
                linked_items: split_comma(linked_items.to_owned().unwrap_or("".to_string())),
                tags: split_comma(tags.to_owned().unwrap_or("".to_string())),
                content: message.to_owned().unwrap_or("".to_string())
            };
            println!("Added new Todo with following Id: {:?}", item.id);
            println!("\"{}\"", item.content);
            let mut items = read_items();
            items.push(item);
            write_items(&items)
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

