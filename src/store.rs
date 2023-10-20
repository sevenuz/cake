pub use self::inner::RecState;
pub use self::inner::Store;
pub use self::inner::MAX_DEPTH;

pub mod inner {
    pub const MAX_DEPTH: usize = 10; /*std::usize::MAX*/
    const MD_DELIMITER: &str = "\n---\n---\n---\n";

    use std::{collections::HashMap, str::FromStr};
    use std::error::Error;

    use crate::item::ParseItemError;
    use crate::{item::Item};
    use serde::{Deserialize, Serialize};

    #[derive(Copy, Clone)]
    pub enum RecState {
        Normal,
        Reappearence,
    }

    #[derive(Copy, Clone)]
    pub struct ItemView<'a> {
        pub item: &'a Item,
        pub depth: usize,
        pub state: RecState,
        pub has_children: bool,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Store {
        items: HashMap<String, Item>,
    }

    impl Store {
        pub fn check_id(&self, id: &String, itself: bool) -> Result<(), &'static str> {
            if itself && !self.items.contains_key(id) {
                return Err("Item does not exist.");
            } else if !itself && self.items.contains_key(id) {
                return Err(
                    "Item already exists. Set the \"edit\" flag if you want to update the item.",
                );
            }
            Ok(())
        }

        pub fn check_existence(&self, item: &Item, itself: bool) -> Result<(), &'static str> {
            self.check_id(&item.id(), itself)?;
            for id in item.children() {
                if !self.items.contains_key(id) {
                    return Err("Not all children exist.");
                }
            }
            for id in item.parents() {
                if !self.items.contains_key(id) {
                    return Err("Not all parents exist.");
                }
            }
            Ok(())
        }

        fn set_relations(&mut self, item: &Item, add: bool) -> Result<(), Box<dyn Error>> {
            for s in item.parents() {
                if let Some(i) = self.get_item_mut(s) {
                    if add {
                        i.add_child(item);
                    } else {
                        i.retain_child(item);
                    }
                }
            }
            for s in item.children() {
                if let Some(i) = self.get_item_mut(s) {
                    if add {
                        i.add_parent(item);
                    } else {
                        i.retain_parent(item);
                    }
                }
            }
            Ok(())
        }

        pub fn new_from_json(file: &str) -> Result<Store, Box<dyn Error>> {
            let serialized = match std::fs::read_to_string(file) {
                Ok(f) => f,
                Err(_err) => {
                    return Ok(Store {
                        items: HashMap::new(),
                    })
                }
            };
            let _store: Store = serde_json::from_str(&serialized)?;
            Ok(_store)
        }

        pub fn write_json(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
            let serialized = serde_json::to_string_pretty(&self)?;
            std::fs::write(file, serialized)?;
            Ok(())
        }

        pub fn new_from_md(file: &str) -> Result<Store, ParseItemError> {
            let serialized = match std::fs::read_to_string(file) {
                Ok(f) => f,
                Err(_err) => {
                    return Ok(Store {
                        items: HashMap::new(),
                    })
                }
            };
            let mut items = HashMap::new();
            let item_strs = serialized.split(MD_DELIMITER);
            for itm_str in item_strs {
                let item = Item::from_str(itm_str)?;
                items.insert(item.id().to_owned(), item);
            }
            let _store: Store = Store { items };
            Ok(_store)
        }

        pub fn write_md(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
            let mut serialized: String = "".to_string();
            for (i, (_id, itm)) in self.items.iter().enumerate() {
                serialized += &itm.print_long(true);
                if i + 1 < self.items.len() {
                    // delimiter of entries
                    serialized += MD_DELIMITER;
                }
            }
            std::fs::write(file, serialized)?;
            Ok(())
        }

        // existence is checked in command
        pub fn edit(&mut self, mut item: Item, overwrite: bool) -> Result<(), Box<dyn Error>> {
            // delete old relations
            self.set_relations(&self.get_item(item.id()).unwrap().clone(), false)?;
            if overwrite {
                self.get_item_mut(item.id()).unwrap().set(item.clone());
            } else {
                self.get_item_mut(item.id()).unwrap().merge(&mut item);
            }
            // set new relations
            self.set_relations(&self.get_item(item.id()).unwrap().clone(), true)?;
            Ok(())
        }

        // existence is checked in command
        pub fn add(&mut self, item: Item) -> Result<(), Box<dyn Error>> {
            self.set_relations(&item, true)?;
            self.items.insert(item.id().to_owned(), item);
            Ok(())
        }

        pub fn remove(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
            match self.items.remove(id) {
                Some(item) => {
                    self.set_relations(&item, false)?;
                }
                None => return Err("Could not found id".into()),
            }
            Ok(())
        }

        pub fn get_item(&self, id: &str) -> Option<&Item> {
            return self.items.get(id);
        }

        pub fn get_item_mut(&mut self, id: &str) -> Option<&mut Item> {
            return self.items.get_mut(id);
        }

        pub fn get(&self) -> &HashMap<String, Item> {
            return &self.items;
        }

        // executes passed fn for every element in items, executing children recursively
        pub fn recursive_execute<'a>(
            &'a self,
            items: &Vec<String>,
            path: &mut Vec<String>,
            depth: usize,
            max_depth: usize,
            up: bool, // recursive for parents
        ) -> Vec<ItemView<'a>> {
            let mut res: Vec<ItemView<'a>> = vec![];
            if depth == max_depth {
                return res;
            }
            for s in items.iter() {
                if let Some(_item) = self.items.get(s) {
                    if !path.contains(&s) {
                        path.push(s.to_string());
                        let mut res1 = vec![];
                        let mut res2 = vec![];
                        if up {
                            res1 = self.recursive_execute(
                                &_item.parents(),
                                path,
                                depth + 1,
                                max_depth,
                                up,
                            );
                        }
                        res.append(&mut res1);
                        res.push(ItemView {
                            item: _item,
                            depth,
                            state: RecState::Normal,
                            has_children: _item.children().len() > 0
                        });
                        if !up {
                            res2 = self.recursive_execute(
                                &_item.children(),
                                path,
                                depth + 1,
                                max_depth,
                                up,
                            );
                        }
                        res.append(&mut res2);
                    } else {
                        res.push(ItemView {
                            item: _item,
                            depth,
                            state: RecState::Reappearence,
                            has_children: _item.children().len() > 0
                        });
                    }
                }
            }
            return res;
        }
    }
}
