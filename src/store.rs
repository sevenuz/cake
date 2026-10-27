pub use self::inner::RecState;
pub use self::inner::Store;
pub use self::inner::MAX_DEPTH;

pub mod inner {
    pub const MAX_DEPTH: usize = 10; /*std::usize::MAX*/

    use std::collections::{HashMap, HashSet};
    use std::error::Error;
    use std::path::{Path, PathBuf};

    use crate::item::Item;

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

    pub struct Store {
        root: PathBuf,
        items: HashMap<String, Item>,
        dirty: HashSet<String>,
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

        fn set_relations(&mut self, item: &Item, add: bool) {
            for s in item.parents() {
                if let Some(i) = self.items.get_mut(s) {
                    if add {
                        i.add_child(item.id());
                    } else {
                        i.retain_child(item.id());
                    }
                }
            }
            for s in item.children() {
                if let Some(i) = self.items.get_mut(s) {
                    if add {
                        if let Some(old) = i.parents().first().cloned() {
                            if let Some(o) = self.items.get_mut(&old) {
                                o.retain_child(s);
                            }
                        }
                        let i = self.items.get_mut(s).unwrap();
                        i.set_parent(Some(item.id().to_owned()));
                    } else {
                        i.set_parent(None);
                    }
                    self.dirty.insert(s.to_owned());
                }
            }
        }

        fn items_dir(root: &Path) -> PathBuf {
            root.join("items")
        }

        /// loads all items/*.md, an empty store if the directory does not exist
        #[cfg(test)]
        pub fn new(root: &Path) -> Result<Store, Box<dyn Error>> {
            Self::new_except(root, &[])
        }

        /// like `new`, but skips the files in `skip` (e.g. conflicted items)
        pub fn new_except(root: &Path, skip: &[PathBuf]) -> Result<Store, Box<dyn Error>> {
            let mut items = HashMap::new();
            let dir = Self::items_dir(root);
            if dir.is_dir() {
                for entry in std::fs::read_dir(&dir)? {
                    let path = entry?.path();
                    if path.extension().map_or(true, |e| e != "md") || skip.contains(&path) {
                        continue;
                    }
                    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let created = std::fs::metadata(&path)?
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64;
                    let item = Item::from_markdown(&std::fs::read_to_string(&path)?, &stem, created)
                        .map_err(|e| format!("{}: {}", path.display(), e))?;
                    items.insert(item.id().to_owned(), item);
                }
            }
            let mut store = Store {
                root: root.to_path_buf(),
                items,
                dirty: HashSet::new(),
            };
            let pairs: Vec<(String, String)> = store
                .items
                .values()
                .flat_map(|i| i.parents().iter().map(|p| (p.to_owned(), i.id().to_owned())))
                .collect();
            for (parent, child) in pairs {
                if let Some(p) = store.items.get_mut(&parent) {
                    p.add_child(&child);
                }
            }
            Ok(store)
        }

        /// writes changed items, removes deleted ones, returns touched paths
        pub fn save(&mut self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
            let dir = Self::items_dir(&self.root);
            if !self.dirty.is_empty() {
                std::fs::create_dir_all(&dir)?;
            }
            let mut paths = vec![];
            for id in self.dirty.drain() {
                let path = dir.join(format!("{}.md", id));
                match self.items.get(&id) {
                    Some(item) => {
                        let tmp = dir.join(format!(".{}.md.tmp", id));
                        std::fs::write(&tmp, item.to_markdown())?;
                        std::fs::rename(&tmp, &path)?;
                    }
                    None if path.exists() => std::fs::remove_file(&path)?,
                    None => continue,
                }
                paths.push(path);
            }
            Ok(paths)
        }

        // existence is checked in command
        pub fn edit(&mut self, mut item: Item, overwrite: bool) -> Result<(), Box<dyn Error>> {
            let old = self.items.get(item.id()).unwrap().clone();
            self.set_relations(&old, false);
            let target = self.get_item_mut(item.id()).unwrap();
            if overwrite {
                target.set(item);
            } else {
                target.merge(&mut item);
            }
            let new = self.items.get(old.id()).unwrap().clone();
            self.set_relations(&new, true);
            Ok(())
        }

        // existence is checked in command
        pub fn add(&mut self, item: Item) -> Result<(), Box<dyn Error>> {
            self.set_relations(&item, true);
            self.dirty.insert(item.id().to_owned());
            self.items.insert(item.id().to_owned(), item);
            Ok(())
        }

        pub fn remove(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
            match self.items.remove(id) {
                Some(item) => {
                    self.set_relations(&item, false);
                    self.dirty.insert(id.to_owned());
                }
                None => return Err("Could not found id".into()),
            }
            Ok(())
        }

        pub fn get_item(&self, id: &str) -> Option<&Item> {
            return self.items.get(id);
        }

        pub fn get_item_mut(&mut self, id: &str) -> Option<&mut Item> {
            self.dirty.insert(id.to_owned());
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
            items: &[String],
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
            Store::new(Path::new("/definitely/does/not/exist/cake-store-test")).unwrap()
        }

        fn item(id: &str, parent: Option<&str>, children: Vec<&str>) -> Item {
            Item::new(
                id.to_string(),
                children.into_iter().map(String::from).collect(),
                parent.map(String::from),
                vec![],
            )
        }

        #[test]
        fn new_returns_empty_store_for_missing_dir() {
            assert!(empty_store().get().is_empty());
        }

        #[test]
        fn add_links_parent_and_derived_child_relations() {
            let mut store = empty_store();
            store.add(item("parent", None, vec![])).unwrap();
            store.add(item("child", Some("parent"), vec![])).unwrap();

            assert!(store
                .get_item("parent")
                .unwrap()
                .children()
                .contains(&"child".to_string()));
        }

        #[test]
        fn add_with_children_sets_their_single_parent() {
            let mut store = empty_store();
            store.add(item("old", None, vec![])).unwrap();
            store.add(item("c", Some("old"), vec![])).unwrap();
            store.add(item("new", None, vec!["c"])).unwrap();

            assert_eq!(store.get_item("c").unwrap().parents(), ["new".to_string()]);
            assert!(store.get_item("old").unwrap().children().is_empty());
        }

        #[test]
        fn remove_unlinks_relations() {
            let mut store = empty_store();
            store.add(item("parent", None, vec![])).unwrap();
            store.add(item("child", Some("parent"), vec![])).unwrap();

            store.remove("child").unwrap();

            assert!(store.get_item("parent").unwrap().children().is_empty());
        }

        #[test]
        fn check_existence_rejects_reference_to_missing_parent() {
            let store = empty_store();
            assert!(store
                .check_existence(&item("orphan", Some("ghost"), vec![]), false)
                .is_err());
        }

        #[test]
        fn check_existence_rejects_duplicate_id_when_not_editing() {
            let mut store = empty_store();
            store.add(item("dup", None, vec![])).unwrap();
            assert!(store.check_existence(&item("dup", None, vec![]), false).is_err());
        }

        #[test]
        fn check_existence_allows_known_id_when_editing() {
            let mut store = empty_store();
            store.add(item("known", None, vec![])).unwrap();
            assert!(store.check_existence(&item("known", None, vec![]), true).is_ok());
        }

        #[test]
        fn edit_without_overwrite_does_not_duplicate_tags() {
            let mut store = empty_store();
            let mut initial = item("x", None, vec![]);
            initial.append_tags(vec!["dup".to_string()]);
            store.add(initial).unwrap();

            let mut update = item("x", None, vec![]);
            update.append_tags(vec!["dup".to_string()]);
            store.edit(update, false).unwrap();

            let tags = store.get_item("x").unwrap().tags();
            assert_eq!(tags.iter().filter(|t| *t == "dup").count(), 1);
        }

        #[test]
        fn recursive_execute_terminates_on_a_cycle() {
            let dir = tempfile::tempdir().unwrap();
            let mut store = Store::new(dir.path()).unwrap();
            store.add(item("a", Some("b"), vec![])).unwrap();
            store.add(item("b", Some("a"), vec![])).unwrap();
            store.save().unwrap();
            let store = Store::new(dir.path()).unwrap();

            let mut path = vec![];
            let views = store.recursive_execute(&["a".to_string()], &mut path, 0, MAX_DEPTH, false);

            assert!(views.len() >= 2);
            assert!(matches!(views.last().unwrap().state, RecState::Reappearence));
        }

        #[test]
        fn recursive_execute_stops_at_max_depth() {
            let mut store = empty_store();
            let n = MAX_DEPTH + 5;
            for i in 0..=n {
                let parent = if i > 0 { Some((i - 1).to_string()) } else { None };
                store
                    .add(Item::new(i.to_string(), vec![], parent, vec![]))
                    .unwrap();
            }

            let mut path = vec![];
            let views = store.recursive_execute(&["0".to_string()], &mut path, 0, MAX_DEPTH, false);

            assert_eq!(views.len(), MAX_DEPTH);
        }

        #[test]
        fn save_and_reload_round_trips_items_and_deletes_removed() {
            let dir = tempfile::tempdir().unwrap();
            let mut store = Store::new(dir.path()).unwrap();
            store.add(item("p", None, vec![])).unwrap();
            store.add(item("c", Some("p"), vec![])).unwrap();
            store.add(item("gone", None, vec![])).unwrap();
            store.get_item_mut("c").unwrap().set_content("# hi\n".to_string());
            store.save().unwrap();
            store.remove("gone").unwrap();
            assert_eq!(store.save().unwrap(), vec![dir.path().join("items/gone.md")]);

            let reloaded = Store::new(dir.path()).unwrap();
            assert_eq!(reloaded.get().len(), 2);
            assert_eq!(reloaded.get_item("p").unwrap().children(), &vec!["c".to_string()]);
            assert_eq!(reloaded.get_item("c").unwrap().to_markdown(), store.get_item("c").unwrap().to_markdown());
        }

        #[test]
        fn foreign_file_is_read_leniently_and_not_rewritten_on_load() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(dir.path().join("items")).unwrap();
            let path = dir.path().join("items/buy milk.md");
            std::fs::write(&path, "buy milk\n").unwrap();

            let mut store = Store::new(dir.path()).unwrap();
            assert_eq!(store.get_item("buy milk").unwrap().content(), "buy milk\n");
            assert!(store.save().unwrap().is_empty());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "buy milk\n");
        }

        #[test]
        fn save_without_changes_creates_nothing() {
            let dir = tempfile::tempdir().unwrap();
            let mut store = Store::new(&dir.path().join("store")).unwrap();
            assert!(store.save().unwrap().is_empty());
            assert!(!dir.path().join("store").exists());
        }

        #[test]
        fn new_except_skips_given_files() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(dir.path().join("items")).unwrap();
            std::fs::write(dir.path().join("items/ok.md"), "ok\n").unwrap();
            let broken = dir.path().join("items/broken.md");
            std::fs::write(&broken, "---\n<<<<<<< HEAD\nid: [\n").unwrap();

            assert!(Store::new(dir.path()).unwrap().get_item("broken").is_some());
            let store = Store::new_except(dir.path(), &[broken]).unwrap();
            assert!(store.get_item("ok").is_some());
            assert!(store.get_item("broken").is_none());
        }
    }
}
