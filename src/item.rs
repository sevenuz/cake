use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub linked_items: Vec<String>,
    pub tags: Vec<String>,
    pub content: String
}
