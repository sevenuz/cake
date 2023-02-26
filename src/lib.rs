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
        _ if matches!(level, Debug::Important) => println!("{}", s.red()),
        2 if matches!(level, Debug::Normal) => println!("{}", s.yellow()),
        _ => (),
    };
    debug(format!("debug mode is on."), Debug::Normal);

    let args: Vec<String> = args().collect();
    debug(format!("{:?}", args), Debug::Normal);

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
            // TODO editor from settings
            let editor = "vim";
            // auto edit flag only works if no flags are used before...
            let _edit = args[1] == "edit" || *edit;
            let _id = &id.to_owned().unwrap_or(generate_id());
            let mut item = Item::new(
                remove_comma(_id.to_string()),
                split_comma(children.to_owned().unwrap_or("".to_string())),
                split_comma(parents.to_owned().unwrap_or("".to_string())),
                split_comma(tags.to_owned().unwrap_or("".to_string())),
                "".to_string()
            );
            store.check_existence(&item, _edit)?;
            item.content = match &message {
                Some(m) => m.to_string(),
                None => input_from_external_editor(
                    editor,
                    if _edit {
                        Some(&store.get_item(&_id.to_string()).unwrap().content)
                    } else {
                        None
                    },
                )
                .unwrap(),
            };
            debug(format!("File content:\n{}", item.content), Debug::Normal);

            if _edit {
                store.edit(item, *overwrite)?;
            } else {
                store.add(item)?;
            }
            println!("{}", store.get_item(_id).unwrap());
            store.write(&file)?;
        }
        Some(Commands::Remove { id, recursive }) => {
            debug(
                format!("id: {:?} recursive: {:?}", id, recursive),
                Debug::Normal,
            );
            store.remove(id, *recursive)?;
            store.write(&file)?;
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
                    store.write(&file)?;
                }
                Err(err) => return Err(err.into()),
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

    Ok(())
}
