mod item;
mod store;
mod util;

use crate::item::{generate_id, Item};
use crate::store::inner::RecState;
use crate::store::Store;
use crate::util::*;
use clap::{Parser, Subcommand};
use colored::*;
use platform_dirs::AppDirs;
use std::{
    env::{args, current_dir, temp_dir},
    error::Error,
    fs::{read_dir, write, File},
    io::Read,
    path::PathBuf,
    process::Command,
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

const NAME: &str = env!("CARGO_PKG_NAME");
const SAVE_FILE: &str = "cake.json";

fn input_from_external_editor(
    editor: &str,
    text: Option<&String>,
) -> Result<String, Box<dyn Error>> {
    let mut file_path = temp_dir();
    file_path.push("editable");
    File::create(&file_path).expect("Could not create file.");

    match text {
        Some(t) => {
            write(&file_path, t).expect("Could not write to file.");
        }
        None => (),
    }

    Command::new(editor)
        .arg(&file_path)
        .status()
        .expect("Something went wrong");

    let mut editable = String::new();
    File::open(file_path)
        .expect("Could not open file")
        .read_to_string(&mut editable)?;
    Ok(editable)
}

// find next cake save file in current or upper dirs, fallback is data_dir
fn find_save_file(path: &mut PathBuf) -> Result<String, Box<dyn Error>> {
    if path.is_dir() {
        for entry in read_dir(path.as_path())? {
            let path = entry?.path();
            let name = path.file_name().ok_or("No filename")?;

            if name == SAVE_FILE {
                return Ok(path.into_os_string().into_string().unwrap());
            }
        }
    }

    if path.pop() {
        return find_save_file(path);
    } else {
        let app_dirs = AppDirs::new(Some(NAME), false).unwrap();
        return Ok(app_dirs
            .data_dir
            .join(SAVE_FILE)
            .into_os_string()
            .into_string()
            .unwrap());
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    enum Debug {
        Important,
        Normal,
    }
    let debug = |s: String, level: Debug| match cli.debug {
        0 => (),
        1 if matches!(level, Debug::Important) => println!("{}", s.red()),
        _ if matches!(level, Debug::Normal) => println!("{}", s.yellow()),
        _ => (),
    };
    debug(format!("debug mode is on."), Debug::Normal);

    // use empty config path to write to global save file
    let file = match cli.config {
        Some(f) => {
            if f.is_empty() {
                find_save_file(&mut PathBuf::new()).unwrap()
            } else {
                f
            }
        }
        None => find_save_file(&mut current_dir()?).unwrap(),
    };
    debug(format!("File: {}", file), Debug::Normal);

    let mut store = Store::new(&file);

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
            // TODO editor from settings
            let editor = "vim";
            let args: Vec<String> = args().collect();
            let cmd_edit = args[1] == "edit";
            debug(format!("{:?}", args), Debug::Normal);
            let _id = &id.to_owned().unwrap_or(generate_id());
            if *edit || cmd_edit {
                let _message = match &message {
                    Some(m) => m.to_string(),
                    None => input_from_external_editor(
                        editor,
                        Some(&store.get_item(&_id.to_string()).unwrap().content),
                    )
                    .unwrap(),
                };
                let item = Item::new(
                    remove_comma(_id.to_string()),
                    split_comma(children.to_owned().unwrap_or("".to_string())),
                    split_comma(parents.to_owned().unwrap_or("".to_string())),
                    split_comma(tags.to_owned().unwrap_or("".to_string())),
                    _message.to_string(),
                );
                store.edit(item, *overwrite).unwrap_or_else(|err| {
                    println!("{}", err);
                });
            } else {
                let _message = match &message {
                    Some(m) => m.to_string(),
                    None => input_from_external_editor(editor, None).unwrap(),
                };
                debug(format!("File content:\n{}", _message), Debug::Normal);
                let item = Item::new(
                    remove_comma(_id.to_string()),
                    split_comma(children.to_owned().unwrap_or("".to_string())),
                    split_comma(parents.to_owned().unwrap_or("".to_string())),
                    split_comma(tags.to_owned().unwrap_or("".to_string())),
                    _message.to_string(),
                );
                store.add(item).unwrap_or_else(|err| {
                    println!("{}", err);
                });
            }
            println!("{}", store.get_item_mut(_id).unwrap());
            store.write(&file);
        }
        Some(Commands::Remove { id, recursive }) => {
            debug(
                format!("id: {:?} recursive: {:?}", id, recursive),
                Debug::Normal,
            );
            store.remove(id, *recursive);
            store.write(&file);
            println!("{:?} removed.", id); // TODO print count of deleted items
        }
        Some(Commands::Start { id }) => {
            match store.get_item_mut(id).expect("Could not found id").start() {
                Ok(_) => {
                    println!("{:?} started.", id); // TODO print count of deleted items
                    store.write(&file);
                }
                Err(err) => println!("{}", err),
            }
        }
        Some(Commands::Stop { id }) => {
            match store.get_item_mut(id).expect("Could not found id").stop() {
                Ok(_) => {
                    println!("{:?} stoped.", id); // TODO print count of deleted items
                    store.write(&file);
                }
                Err(err) => println!("{}", err),
            }
        }
        Some(Commands::List {
            id,
            recursive,
            long,
        }) => {
            debug(
                format!("id: {:?} recursive: {:?} long: {:?}", id, recursive, long),
                Debug::Important,
            );
            let _id = &id.to_owned().unwrap_or("".to_string());
            let _items = store.get();
            let mut _cycle: Vec<String> = vec![];
            let max_depth = if *recursive {
                10 /*std::usize::MAX*/
            } else {
                1
            };

            fn prl(item: &Item, depth: usize, state: RecState) {
                println!(
                    "{:indent$}{}",
                    "",
                    match state {
                        RecState::Normal => item.to_string().bold(),
                        RecState::Cycle => (item.to_string() + " (cycle)").purple(),
                        RecState::Reappearence =>
                            (item.to_string() + " (reappearence)").bright_green(),
                    },
                    indent = depth
                );
            }
            fn prs(item: &Item, depth: usize, state: RecState) {
                println!(
                    "{:indent$}{}",
                    "",
                    match state {
                        RecState::Normal => item.print().bold(),
                        RecState::Cycle => (item.print() + " (cycle)").purple(),
                        RecState::Reappearence => (item.print() + " (reappearence)").bright_green(),
                    },
                    indent = depth
                );
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
                // TODO
                // run separatly per key
                // remove from keys entries from cycle
                store.recursive_execute(
                    &_keys,
                    &mut _cycle,
                    if *long { prl } else { prs },
                    0,
                    max_depth,
                );
            } else {
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

    Ok(())
}
