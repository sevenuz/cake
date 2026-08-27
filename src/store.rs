pub use self::inner::RecState;
pub use self::inner::Store;
pub use self::inner::MAX_DEPTH;

pub mod inner {
    pub const MAX_DEPTH: usize = 10; /*std::usize::MAX*/
    const MD_DELIMITER: &str = "\n---\n---\n---\n";
    // Stand-in for MD_DELIMITER when it shows up inside an item's own content,
    // so splitting the file on MD_DELIMITER can't mistake it for an item
    // boundary. Distinguished by a zero-width space real content won't
    // plausibly contain at that exact position.
    const MD_DELIMITER_ESCAPED: &str = "\n-\u{200B}--\n---\n---\n";

    use std::error::Error;
    use std::{collections::HashMap, str::FromStr};

    use crate::item::Item;
    use crate::error::ParseError;
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

        pub fn new_from_md(file: &str) -> Result<Store, ParseError> {
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
                if itm_str.is_empty() {
                    continue;
                }
                let unescaped = itm_str.replace(MD_DELIMITER_ESCAPED, MD_DELIMITER);
                let item = Item::from_str(&unescaped)?;
                items.insert(item.id().to_owned(), item);
            }
            let _store: Store = Store { items };
            Ok(_store)
        }

        pub fn write_md(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
            let mut serialized: String = "".to_string();

            let mut keys = self.items.keys().cloned().collect::<Vec<String>>();
            // sort output from old to new
            keys.sort_by(|a, b| {
                self.items
                    .get(a)
                    .unwrap()
                    .timestamp()
                    .cmp(&self.items.get(b).unwrap().timestamp())
            });
            for (i, id) in keys.iter().enumerate() {
                let text = self.get_item(id).unwrap().print_long(true);
                serialized += &text.replace(MD_DELIMITER, MD_DELIMITER_ESCAPED);
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

        /// generates an ItemView for every element in items, executing children recursively
        /// ids are collected in path
        /// @up to recursivly go through parents
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
                            has_children: _item.children().len() > 0,
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
                            has_children: _item.children().len() > 0,
                        });
                    }
                }
            }
            return res;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn empty_store() -> Store {
            Store::new_from_json("/definitely/does/not/exist/cake-store-test.json").unwrap()
        }

        fn item(id: &str, parents: Vec<&str>, children: Vec<&str>) -> Item {
            Item::new(
                id.to_string(),
                children.into_iter().map(String::from).collect(),
                parents.into_iter().map(String::from).collect(),
                vec![],
            )
        }

        #[test]
        fn new_from_json_returns_empty_store_for_missing_file() {
            let store =
                Store::new_from_json("/definitely/does/not/exist/cake-store-test.json").unwrap();
            assert!(store.get().is_empty());
        }

        #[test]
        fn new_from_md_returns_empty_store_for_missing_file() {
            let store =
                Store::new_from_md("/definitely/does/not/exist/cake-store-test.md").unwrap();
            assert!(store.get().is_empty());
        }

        #[test]
        fn add_links_reciprocal_parent_and_child_relations() {
            let mut store = empty_store();
            store.add(item("parent", vec![], vec![])).unwrap();
            store.add(item("child", vec!["parent"], vec![])).unwrap();

            assert!(store
                .get_item("parent")
                .unwrap()
                .children()
                .contains(&"child".to_string()));
        }

        #[test]
        fn remove_unlinks_reciprocal_relations() {
            let mut store = empty_store();
            store.add(item("parent", vec![], vec![])).unwrap();
            store.add(item("child", vec!["parent"], vec![])).unwrap();

            store.remove("child").unwrap();

            assert!(!store
                .get_item("parent")
                .unwrap()
                .children()
                .contains(&"child".to_string()));
        }

        #[test]
        fn check_existence_rejects_reference_to_missing_parent() {
            let store = empty_store();
            let orphan = item("orphan", vec!["ghost"], vec![]);
            assert!(store.check_existence(&orphan, false).is_err());
        }

        #[test]
        fn check_existence_rejects_duplicate_id_when_not_editing() {
            let mut store = empty_store();
            store.add(item("dup", vec![], vec![])).unwrap();
            assert!(store
                .check_existence(&item("dup", vec![], vec![]), false)
                .is_err());
        }

        #[test]
        fn check_existence_allows_known_id_when_editing() {
            let mut store = empty_store();
            store.add(item("known", vec![], vec![])).unwrap();
            assert!(store
                .check_existence(&item("known", vec![], vec![]), true)
                .is_ok());
        }

        #[test]
        fn edit_without_overwrite_can_duplicate_an_already_present_tag() {
            let mut store = empty_store();
            let mut initial = item("x", vec![], vec![]);
            initial.append_tags(vec!["dup".to_string()]);
            store.add(initial).unwrap();

            let mut update = item("x", vec![], vec![]);
            update.append_tags(vec!["dup".to_string()]);
            store.edit(update, false).unwrap();

            // BUG: Item::merge appends tags unconditionally (unlike
            // append_tags, which dedupes), so editing with a tag that's
            // already present duplicates it instead of leaving it alone.
            let tags = store.get_item("x").unwrap().tags();
            assert_eq!(tags.iter().filter(|t| *t == "dup").count(), 1);
        }

        #[test]
        fn recursive_execute_terminates_on_a_cycle() {
            let mut store = empty_store();
            // children-graph cycle: a -> b -> a
            store.add(item("a", vec![], vec!["b"])).unwrap();
            store.add(item("b", vec![], vec!["a"])).unwrap();

            let keys = vec!["a".to_string()];
            let mut path = vec![];
            let views = store.recursive_execute(&keys, &mut path, 0, MAX_DEPTH, false);

            assert!(views.len() >= 2);
            assert!(matches!(views.last().unwrap().state, RecState::Reappearence));
        }

        #[test]
        fn recursive_execute_stops_at_max_depth() {
            let mut store = empty_store();
            let n = MAX_DEPTH + 5;
            for i in 0..=n {
                let children = if i < n {
                    vec![(i + 1).to_string()]
                } else {
                    vec![]
                };
                store
                    .add(Item::new(i.to_string(), children, vec![], vec![]))
                    .unwrap();
            }

            let keys = vec!["0".to_string()];
            let mut path = vec![];
            let views = store.recursive_execute(&keys, &mut path, 0, MAX_DEPTH, false);

            assert_eq!(views.len(), MAX_DEPTH);
        }

        #[test]
        fn json_and_markdown_formats_round_trip_to_equivalent_items() {
            let dir = tempfile::tempdir().unwrap();
            let json_path = dir.path().join("cake.json");
            let md_path = dir.path().join("cake.md");

            let mut store = empty_store();
            store.add(item("root", vec![], vec![])).unwrap();
            let mut child = item("kid", vec!["root"], vec![]);
            child.append_tags(vec!["urgent".to_string(), "backend".to_string()]);
            child.set_content("multi\nline\ncontent".to_string());
            store.add(child).unwrap();

            store.write_json(json_path.to_str().unwrap()).unwrap();
            store.write_md(md_path.to_str().unwrap()).unwrap();

            let from_json = Store::new_from_json(json_path.to_str().unwrap()).unwrap();
            let from_md = Store::new_from_md(md_path.to_str().unwrap()).unwrap();

            let mut json_ids: Vec<&String> = from_json.get().keys().collect();
            let mut md_ids: Vec<&String> = from_md.get().keys().collect();
            json_ids.sort();
            md_ids.sort();
            assert_eq!(json_ids, md_ids);

            for id in json_ids {
                assert_eq!(
                    from_json.get_item(id).unwrap().print_long(true),
                    from_md.get_item(id).unwrap().print_long(true),
                );
            }
        }

        #[test]
        fn markdown_round_trip_preserves_content_containing_the_item_delimiter() {
            let dir = tempfile::tempdir().unwrap();
            let md_path = dir.path().join("cake.md");

            let mut store = empty_store();
            let mut poison = item("poison", vec![], vec![]);
            // The markdown store separates items with the literal sequence
            // "\n---\n---\n---\n" (MD_DELIMITER above). If that sequence shows
            // up inside an item's own content, it should not be mistaken for
            // a boundary between two items.
            poison.set_content("before\n---\n---\n---\nafter".to_string());
            store.add(poison).unwrap();

            store.write_md(md_path.to_str().unwrap()).unwrap();
            let reloaded = Store::new_from_md(md_path.to_str().unwrap()).unwrap();

            assert_eq!(reloaded.get().len(), 1);
            assert_eq!(
                reloaded.get_item("poison").unwrap().content(),
                "before\n---\n---\n---\nafter",
            );
        }
    }
}
