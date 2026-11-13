pub use self::inner::{init_weights, ScoreWeights, Store};
pub use self::inner::MAX_DEPTH;

pub mod inner {
    pub const MAX_DEPTH: usize = 10; /*std::usize::MAX*/

    use std::collections::{HashMap, HashSet};
    use std::error::Error;
    use std::path::{Path, PathBuf};

    use crate::error;
    use crate::item::{Item, Status};
    use crate::util;
    use serde::{Deserialize, Serialize};
    use std::sync::OnceLock;

    /// points of the score, config `score`
    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(default)]
    pub struct ScoreWeights {
        pub deadline_today: i64,
        pub deadline_per_day: i64,
        pub deadline_max: i64,
        pub due_today: i64,
        pub due_per_day: i64,
        pub due_max: i64,
        pub per_waiting: i64,
        pub prio: [i64; 3],
        pub age_max: i64,
    }

    impl Default for ScoreWeights {
        fn default() -> Self {
            ScoreWeights {
                deadline_today: 100,
                deadline_per_day: 10,
                deadline_max: 200,
                due_today: 50,
                due_per_day: 5,
                due_max: 100,
                per_waiting: 10,
                prio: [30, 20, 10],
                age_max: 20,
            }
        }
    }

    static WEIGHTS: OnceLock<ScoreWeights> = OnceLock::new();

    pub fn init_weights(w: ScoreWeights) {
        let _ = WEIGHTS.set(w);
    }

    #[derive(Copy, Clone)]
    pub enum RecState {
        Normal,
        Reappearence,
    }

    /// collected by recursive_execute, used for the visited path
    #[allow(dead_code)]
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
        /// where each item currently is on disk
        paths: HashMap<String, PathBuf>,
    }

    impl Store {
        pub fn check_id(&self, id: &String, itself: bool) -> Result<(), Box<dyn Error>> {
            if itself && !self.items.contains_key(id) {
                return Err(error::not_found(id));
            } else if !itself && self.items.contains_key(id) {
                return Err(error::invalid(format!("{} already exists, use cake edit.", id)));
            }
            Ok(())
        }

        pub fn check_existence(&self, item: &Item, itself: bool) -> Result<(), Box<dyn Error>> {
            self.check_id(&item.id(), itself)?;
            for id in item.children() {
                if !self.items.contains_key(id) {
                    return Err(error::not_found(id));
                }
            }
            for id in item.parents() {
                if !self.items.contains_key(id) {
                    return Err(error::not_found(id));
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

        /// loads all items/**/*.md, an empty store if the directory does not exist
        #[cfg(test)]
        pub fn new(root: &Path) -> Result<Store, Box<dyn Error>> {
            Self::new_except(root, &[])
        }

        /// like `new`, but skips the files in `skip` (e.g. conflicted items)
        pub fn new_except(root: &Path, skip: &[PathBuf]) -> Result<Store, Box<dyn Error>> {
            let mut items = HashMap::new();
            let mut paths = HashMap::new();
            let dir = Self::items_dir(root);
            if dir.is_dir() {
                Self::load_dir(&dir, None, skip, &mut items, &mut paths)?;
            }
            let mut store = Store {
                root: root.to_path_buf(),
                items,
                dirty: HashSet::new(),
                paths,
            };
            // a parent that does not exist (e.g. a directory left by a merge) detaches
            let dangling: Vec<String> = store
                .items
                .values()
                .filter(|i| i.parents().first().is_some_and(|p| !store.items.contains_key(p)))
                .map(|i| i.id().to_owned())
                .collect();
            for id in dangling {
                store.items.get_mut(&id).unwrap().set_parent(None);
            }
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

        /// loads `dir` recursively; inside `<parent>/` the directory decides the parent
        fn load_dir(
            dir: &Path,
            parent: Option<&str>,
            skip: &[PathBuf],
            items: &mut HashMap<String, Item>,
            paths: &mut HashMap<String, PathBuf>,
        ) -> Result<(), Box<dyn Error>> {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if name.starts_with('.') || skip.contains(&path) {
                    continue;
                }
                if path.is_dir() {
                    Self::load_dir(&path, Some(&name), skip, items, paths)?;
                    continue;
                }
                if path.extension().map_or(true, |e| e != "md") {
                    continue;
                }
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let created = std::fs::metadata(&path)?
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64;
                let mut item = Item::from_markdown(&std::fs::read_to_string(&path)?, &stem, created)
                    .map_err(|e| format!("{}: {}", path.display(), e))?;
                if let Some(p) = parent {
                    item.set_parent(Some(p.to_owned()));
                }
                paths.insert(item.id().to_owned(), path);
                items.insert(item.id().to_owned(), item);
            }
            Ok(())
        }

        /// items/<ancestors>/<id>.md, unknown ancestors and cycles end the chain
        fn item_path(&self, id: &str) -> PathBuf {
            let mut chain = vec![];
            let mut cur = self.items.get(id).and_then(|i| i.parents().first());
            while let Some(p) = cur {
                if p == id || chain.contains(&p) || !self.items.contains_key(p) {
                    break;
                }
                chain.push(p);
                cur = self.items.get(p).and_then(|i| i.parents().first());
            }
            let mut path = Self::items_dir(&self.root);
            for p in chain.iter().rev() {
                path.push(p);
            }
            path.join(format!("{}.md", id))
        }

        /// removes empty directories from `dir` up to the items directory
        fn prune(&self, dir: Option<&Path>) {
            let top = Self::items_dir(&self.root);
            let mut dir = dir;
            while let Some(d) = dir {
                if d == top || !d.starts_with(&top) || std::fs::remove_dir(d).is_err() {
                    break;
                }
                dir = d.parent();
            }
        }

        /// records `session` on every changed item
        pub fn add_session(&mut self, session: &str) {
            for id in &self.dirty {
                if let Some(i) = self.items.get_mut(id) {
                    i.add_session(session);
                }
            }
        }

        /// writes changed items, removes deleted ones and moves files whose
        /// parent changed (with their subtree); returns touched paths
        pub fn save(&mut self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
            if self.dirty.is_empty() {
                return Ok(vec![]);
            }
            let mut touched = vec![];
            let dirty = std::mem::take(&mut self.dirty);
            for id in dirty.iter().filter(|id| !self.items.contains_key(*id)) {
                if let Some(old) = self.paths.remove(id) {
                    if old.exists() {
                        std::fs::remove_file(&old)?;
                        self.prune(old.parent());
                    }
                    touched.push(old);
                }
            }
            let mut ids: Vec<String> = self.items.keys().cloned().collect();
            ids.sort();
            for id in ids {
                let want = self.item_path(&id);
                let old = self.paths.get(&id).cloned();
                let changed = dirty.contains(&id);
                if !changed && old.as_ref() == Some(&want) {
                    continue;
                }
                let dir = want.parent().unwrap();
                std::fs::create_dir_all(dir)?;
                match &old {
                    Some(o) if !changed && o.exists() => std::fs::rename(o, &want)?,
                    _ => {
                        let tmp = dir.join(format!(".{}.md.tmp", id));
                        std::fs::write(&tmp, self.items[&id].to_markdown())?;
                        std::fs::rename(&tmp, &want)?;
                    }
                }
                if let Some(o) = old.filter(|o| *o != want) {
                    if o.exists() {
                        std::fs::remove_file(&o)?;
                    }
                    self.prune(o.parent());
                    touched.push(o);
                }
                self.paths.insert(id, want.clone());
                touched.push(want);
            }
            Ok(touched)
        }

        /// replaces an existing item, keeps derived children; existence is checked in command
        pub fn edit(&mut self, item: Item) -> Result<(), Box<dyn Error>> {
            let old = self.items.get(item.id()).unwrap().clone();
            self.set_relations(&old, false);
            self.get_item_mut(item.id()).unwrap().set(item);
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
                None => return Err(error::not_found(id)),
            }
            Ok(())
        }

        /// true if `ancestor` is `id` or one of its parents, transitively
        fn is_ancestor(&self, ancestor: &str, id: &str) -> bool {
            let mut seen = HashSet::new();
            let mut cur = Some(id.to_owned());
            while let Some(c) = cur {
                if c == ancestor {
                    return true;
                }
                if !seen.insert(c.clone()) {
                    return false;
                }
                cur = self.items.get(&c).and_then(|i| i.parents().first().cloned());
            }
            false
        }

        /// re-parents `id` under `parent`, None detaches; refuses cycles
        pub fn move_item(&mut self, id: &str, parent: Option<&str>) -> Result<(), Box<dyn Error>> {
            if !self.items.contains_key(id) {
                return Err(error::not_found(id));
            }
            if let Some(p) = parent {
                if !self.items.contains_key(p) {
                    return Err(error::not_found(p));
                }
                if self.is_ancestor(id, p) {
                    return Err(error::invalid(format!("{} is {} or below it, that would be a cycle.", p, id)));
                }
            }
            let item = self.items.get(id).unwrap().clone();
            self.set_relations(&item, false);
            let item = self.get_item_mut(id).unwrap();
            item.set_parent(parent.map(|p| p.to_owned()));
            let item = item.clone();
            self.set_relations(&item, true);
            Ok(())
        }

        /// true if `from` waits on `target`, directly or transitively
        fn waits_on(&self, from: &str, target: &str) -> bool {
            let mut seen = HashSet::new();
            let mut todo = vec![from.to_owned()];
            while let Some(c) = todo.pop() {
                if c == target {
                    return true;
                }
                if seen.insert(c.clone()) {
                    if let Some(i) = self.items.get(&c) {
                        todo.extend(i.blocked_by().iter().cloned());
                    }
                }
            }
            false
        }

        /// adds blockers to `id`, refuses unknown ids and cycles
        pub fn block(&mut self, id: &str, blockers: &[String]) -> Result<(), Box<dyn Error>> {
            for b in std::iter::once(&id.to_owned()).chain(blockers) {
                if !self.items.contains_key(b) {
                    return Err(error::not_found(b));
                }
            }
            for b in blockers {
                if self.waits_on(b, id) {
                    return Err(error::invalid(format!("{} waits on {}, that would be a cycle.", b, id)));
                }
                // a parent waits for its work children: blocking a child by it would deadlock
                if b != id && self.is_ancestor(b, id) {
                    return Err(error::invalid(format!("{} is above {} and waits for it; block by a sibling instead.", b, id)));
                }
            }
            let item = self.get_item_mut(id).unwrap();
            let mut ids = item.blocked_by().clone();
            ids.extend(blockers.iter().filter(|b| !item.blocked_by().contains(b)).cloned());
            item.set_blocked_by(ids);
            Ok(())
        }

        pub fn unblock(&mut self, id: &str, blockers: &[String]) -> Result<(), Box<dyn Error>> {
            let item = self.get_item_mut(id).ok_or_else(|| error::not_found(id))?;
            let ids = item.blocked_by().iter().filter(|b| !blockers.contains(b)).cloned().collect();
            item.set_blocked_by(ids);
            Ok(())
        }

        /// open items waiting on `id`, directly or transitively
        pub fn waiting_on(&self, id: &str) -> usize {
            let mut seen = HashSet::new();
            let mut todo = vec![id.to_owned()];
            while let Some(c) = todo.pop() {
                for i in self.items.values().filter(|i| i.blocked_by().contains(&c)) {
                    if i.is_open() && seen.insert(i.id().to_owned()) {
                        todo.push(i.id().to_owned());
                    }
                }
            }
            seen.len()
        }

        /// how much an open item needs attention, 0 for closed ones: deadline
        /// pressure (hard and soft), items waiting on it, priority and age,
        /// weighted by config `score`
        pub fn score(&self, id: &str, now: i64) -> i64 {
            let Some(i) = self.items.get(id).filter(|i| i.status().map_or(true, |s| s.is_open())) else {
                return 0;
            };
            let w = WEIGHTS.get_or_init(ScoreWeights::default);
            let days = |d: Option<&String>| d.and_then(|d| util::date_timestamp(d)).map(|t| (t - now) / 86400);
            let mut score = 0;
            if let Some(d) = days(i.deadline()) {
                score += (w.deadline_today - w.deadline_per_day * d).clamp(0, w.deadline_max);
            }
            if let Some(d) = days(i.due()) {
                score += (w.due_today - w.due_per_day * d).clamp(0, w.due_max);
            }
            score += w.per_waiting * self.waiting_on(id) as i64;
            score += match i.priority() {
                Some(p @ 1..=3) => w.prio[p as usize - 1],
                _ => 0,
            };
            score + ((now - i.timestamp()) / 86400).clamp(0, w.age_max)
        }

        /// open item with at least one open blocker
        pub fn is_blocked(&self, id: &str) -> bool {
            self.items.get(id).is_some_and(|i| {
                i.is_open()
                    && i.blocked_by()
                        .iter()
                        .any(|b| self.items.get(b).is_some_and(|b| b.is_open()))
            })
        }

        /// the parts of `id`'s work: children with a status that do not block it
        /// (a question under a task blocks it, it is no part of its work)
        fn work_children(&self, id: &str) -> Vec<&Item> {
            self.items.get(id).map_or(vec![], |i| {
                i.children()
                    .iter()
                    .filter(|c| !i.blocked_by().contains(c))
                    .filter_map(|c| self.items.get(c))
                    .filter(|c| c.status().is_some())
                    .collect()
            })
        }

        /// open work below `id`
        pub fn open_children(&self, id: &str) -> usize {
            self.work_children(id).iter().filter(|c| c.is_open()).count()
        }

        /// a task (or started one) without open blockers and open children
        pub fn is_ready(&self, id: &str) -> bool {
            self.items.get(id).is_some_and(|i| matches!(i.status(), Some(Status::Task) | Some(Status::Doing)))
                && !self.is_blocked(id)
                && self.open_children(id) == 0
        }

        /// tasks whose work children are all closed go to review by themselves
        pub fn settle(&mut self) -> Vec<String> {
            let ids: Vec<String> = self
                .items
                .values()
                .filter(|i| matches!(i.status(), Some(Status::Task) | Some(Status::Doing)))
                .filter(|i| {
                    let work = self.work_children(i.id());
                    !work.is_empty() && work.iter().all(|c| !c.is_open())
                })
                .map(|i| i.id().to_owned())
                .collect();
            for id in &ids {
                let _ = self.get_item_mut(id).unwrap().set_status(Status::Review);
            }
            ids
        }

        /// item json with derived blocks, ready and path
        pub fn item_json(&self, id: &str) -> Option<serde_json::Value> {
            let item = self.items.get(id)?;
            let mut m = item.to_json();
            let mut blocks: Vec<&String> = self
                .items
                .values()
                .filter(|i| i.blocked_by().iter().any(|b| b == id))
                .map(|i| i.id())
                .collect();
            blocks.sort();
            m.insert("blocks".into(), serde_json::json!(blocks));
            m.insert("ready".into(), self.is_ready(id).into());
            m.insert("score".into(), self.score(id, util::timestamp()).into());
            let path = self.item_path(id);
            m.insert("path".into(), path.to_string_lossy().into());
            Some(serde_json::Value::Object(m))
        }

        /// full id for an exact id or a unique prefix (at least 2 chars);
        /// unknown input is returned unchanged, so the caller reports not_found
        pub fn resolve(&self, input: &str) -> Result<String, Box<dyn Error>> {
            if self.items.contains_key(input) || input.chars().count() < 2 {
                return Ok(input.to_string());
            }
            let mut matches: Vec<&String> = self.items.keys().filter(|k| k.starts_with(input)).collect();
            matches.sort();
            match matches.len() {
                0 => Ok(input.to_string()),
                1 => Ok(matches[0].clone()),
                _ => Err(error::coded(
                    "ambiguous_id",
                    format!("{} matches {}", input, matches.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", ")),
                )),
            }
        }

        /// resolves every id of a comma separated list
        pub fn resolve_list(&self, input: &str) -> Result<String, Box<dyn Error>> {
            Ok(input.split(',').map(|i| self.resolve(i.trim())).collect::<Result<Vec<_>, _>>()?.join(","))
        }

        /// shortest unique prefix of every id, at least 2 chars
        pub fn short_ids(&self) -> HashMap<String, String> {
            let mut ids: Vec<&String> = self.items.keys().collect();
            ids.sort();
            let common = |a: &str, b: &str| a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
            (0..ids.len())
                .map(|n| {
                    let before = if n > 0 { common(ids[n - 1], ids[n]) } else { 0 };
                    let after = ids.get(n + 1).map_or(0, |next| common(ids[n], next));
                    let len = (before.max(after) + 1).max(2);
                    (ids[n].clone(), ids[n].chars().take(len).collect())
                })
                .collect()
        }

        pub fn root(&self) -> &Path {
            &self.root
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
        fn edit_replaces_item_and_keeps_children() {
            let mut store = empty_store();
            store.add(item("p", None, vec![])).unwrap();
            store.add(item("c", Some("p"), vec![])).unwrap();

            let mut update = item("p", None, vec![]);
            update.append_tags(vec!["new".to_string()]);
            store.edit(update).unwrap();

            let p = store.get_item("p").unwrap();
            assert_eq!(p.tags(), &vec!["new".to_string()]);
            assert_eq!(p.children(), &vec!["c".to_string()]);
            assert_eq!(store.get_item("c").unwrap().parents(), ["p".to_string()]);
        }

        #[test]
        fn move_item_reparents_and_detaches() {
            let mut store = empty_store();
            store.add(item("a", None, vec![])).unwrap();
            store.add(item("b", None, vec![])).unwrap();
            store.add(item("c", Some("a"), vec![])).unwrap();

            store.move_item("c", Some("b")).unwrap();
            assert!(store.get_item("a").unwrap().children().is_empty());
            assert_eq!(store.get_item("b").unwrap().children(), &vec!["c".to_string()]);
            assert_eq!(store.get_item("c").unwrap().parents(), ["b".to_string()]);

            store.move_item("c", None).unwrap();
            assert!(store.get_item("b").unwrap().children().is_empty());
            assert!(store.get_item("c").unwrap().parents().is_empty());
        }

        #[test]
        fn move_item_refuses_cycles_and_unknown_ids() {
            let mut store = empty_store();
            store.add(item("a", None, vec![])).unwrap();
            store.add(item("b", Some("a"), vec![])).unwrap();
            store.add(item("c", Some("b"), vec![])).unwrap();

            assert!(store.move_item("a", Some("c")).is_err());
            assert!(store.move_item("a", Some("a")).is_err());
            assert!(store.move_item("a", Some("x")).is_err());
            assert!(store.move_item("x", None).is_err());
            assert!(store.get_item("a").unwrap().parents().is_empty());
        }

        #[test]
        fn block_adds_once_and_refuses_cycles() {
            let mut store = empty_store();
            for id in ["a", "b", "c"] {
                store.add(item(id, None, vec![])).unwrap();
            }
            store.block("a", &["b".into()]).unwrap();
            store.block("a", &["b".into(), "c".into()]).unwrap();
            store.block("b", &["c".into()]).unwrap();
            assert_eq!(store.get_item("a").unwrap().blocked_by(), &vec!["b".to_string(), "c".to_string()]);

            assert!(store.block("c", &["a".into()]).is_err(), "c <- b <- a");
            assert!(store.block("a", &["a".into()]).is_err());
            assert!(store.block("a", &["x".into()]).is_err());

            store.unblock("a", &["b".into()]).unwrap();
            assert_eq!(store.get_item("a").unwrap().blocked_by(), &vec!["c".to_string()]);
        }

        #[test]
        fn prefixes_resolve_and_short_ids_are_unique() {
            let mut store = empty_store();
            for id in ["abcdef", "abxyzq", "kmnpqr", "x"] {
                store.add(item(id, None, vec![])).unwrap();
            }
            assert_eq!(store.resolve("km").unwrap(), "kmnpqr");
            assert_eq!(store.resolve("abc").unwrap(), "abcdef");
            assert_eq!(error::code(store.resolve("ab").unwrap_err().as_ref()), "ambiguous_id");
            assert_eq!(store.resolve("k").unwrap(), "k", "too short, stays unknown");
            assert_eq!(store.resolve("x").unwrap(), "x", "exact match");
            assert_eq!(store.resolve_list("km, abx").unwrap(), "kmnpqr,abxyzq");
            let short = store.short_ids();
            assert_eq!((short["abcdef"].as_str(), short["abxyzq"].as_str()), ("abc", "abx"));
            assert_eq!((short["kmnpqr"].as_str(), short["x"].as_str()), ("km", "x"));
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
        fn children_live_in_a_directory_next_to_the_parent_and_move_with_it() {
            let dir = tempfile::tempdir().unwrap();
            let items = dir.path().join("items");
            let mut store = Store::new(dir.path()).unwrap();
            store.add(item("p", None, vec![])).unwrap();
            store.add(item("c", Some("p"), vec![])).unwrap();
            store.add(item("g", Some("c"), vec![])).unwrap();
            store.add(item("q", None, vec![])).unwrap();
            store.save().unwrap();
            assert!(items.join("p.md").is_file());
            assert!(items.join("p/c/g.md").is_file());

            store.move_item("c", Some("q")).unwrap();
            let mut touched = store.save().unwrap();
            touched.sort();
            assert_eq!(
                touched,
                vec![items.join("p/c/g.md"), items.join("p/c.md"), items.join("q/c/g.md"), items.join("q/c.md")]
            );
            assert!(!items.join("p").exists(), "empty directory is pruned");
            assert!(items.join("q/c/g.md").is_file());

            let reloaded = Store::new(dir.path()).unwrap();
            assert_eq!(reloaded.get_item("g").unwrap().parents(), &["c".to_string()]);
            assert_eq!(reloaded.get_item("q").unwrap().children(), &vec!["c".to_string()]);
        }

        #[test]
        fn directory_decides_the_parent_and_flat_stores_migrate_on_write() {
            let dir = tempfile::tempdir().unwrap();
            let items = dir.path().join("items");
            std::fs::create_dir_all(items.join("p")).unwrap();
            std::fs::write(items.join("p.md"), "---\nid: p\n---\np\n").unwrap();
            std::fs::write(items.join("old.md"), "---\nid: old\nparent: '[[p]]'\n---\nold\n").unwrap();
            std::fs::write(items.join("p/moved.md"), "---\nid: moved\n---\nmoved\n").unwrap();
            std::fs::create_dir_all(items.join("orphan")).unwrap();
            std::fs::write(items.join("orphan/lost.md"), "---\nid: lost\n---\nlost\n").unwrap();

            let mut store = Store::new(dir.path()).unwrap();
            assert_eq!(store.get_item("moved").unwrap().parents(), &["p".to_string()]);
            assert!(store.get_item("lost").unwrap().parents().is_empty());
            assert!(store.save().unwrap().is_empty(), "reads write nothing");

            store.get_item_mut("p").unwrap();
            store.save().unwrap();
            assert!(items.join("p/old.md").is_file());
            assert!(items.join("lost.md").is_file());
            assert!(!items.join("orphan").exists());
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
