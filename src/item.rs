use serde::{Deserialize, Serialize};
use nanoid::nanoid;

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub linked_items: Vec<String>,
    pub tags: Vec<String>,
    pub content: String
}

pub fn generate_id() -> String {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
    ];

    nanoid!(3, &alphabet)
}

pub fn write_items(items: &Vec<Item>) {
    // TODO configure file
    let serialized = serde_json::to_string_pretty(&items).unwrap();
    std::fs::write("./cake.json", serialized).unwrap();
}

pub fn read_items() -> Vec<Item> {
    // TODO configure file
    // TODO return empty Vec directly instead of parsing []...
    let serialized = std::fs::read_to_string("./cake.json").unwrap_or("[]".to_string());
    let _items: Vec<Item> = serde_json::from_str(&serialized).unwrap();
    _items
}

impl Item {
}
