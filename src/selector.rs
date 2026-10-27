use crate::item::{Kind, Status};
use crate::store::{Store, MAX_DEPTH};
use crate::util;
use std::error::Error;

// default filter
#[derive(Debug)]
pub struct Selector {
    pub ids: Vec<String>,
    pub children: Vec<String>,
    pub parents: Vec<String>,
    pub tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub before: Option<i64>, // time in seconds relative to now
    pub after: Option<i64>,  // time in seconds relative to now
    pub started: bool,
    pub stopped: bool,
    pub rparents: bool,  // recursive for parents
    pub rchildren: bool, // recursive for children
    or: bool,            // use or concatination of selectors
    pub search: Option<String>, // lowercase substring of the content
    pub kind: Option<Kind>,
    pub status: Vec<Status>,
    pub ready: bool,
    pub blocked: bool,
}

impl Selector {
    pub fn new(
        ids: &Option<String>,
        children: &Option<String>,
        parents: &Option<String>,
        tags: &Option<String>,
        before: &Option<String>,
        after: &Option<String>,
        started: &bool,
        stopped: &bool,
        recursive: &u8,
        or: &bool,
    ) -> Result<Selector, Box<dyn Error>> {
        Ok(Selector {
            ids: util::split_comma_cleanup(ids.to_owned().unwrap_or("".to_string())),
            children: util::split_comma_cleanup(children.to_owned().unwrap_or("".to_string())),
            parents: util::split_comma_cleanup(parents.to_owned().unwrap_or("".to_string())),
            tags: util::split_comma_tags(tags.to_owned().unwrap_or("".to_string())),
            exclude_tags: util::split_comma_exclude_tags(tags.to_owned().unwrap_or("".to_string())),
            before: util::parse_time(&before.to_owned().unwrap_or("".to_string()))?,
            after: util::parse_time(&after.to_owned().unwrap_or("".to_string()))?,
            started: *started,
            stopped: *stopped,
            rparents: *recursive > 1, // -rr only parents, -rrr both
            rchildren: *recursive == 1 || *recursive > 2, // -r only children, -rrr both
            or: *or,
            search: None,
            kind: None,
            status: vec![],
            ready: false,
            blocked: false,
        })
    }

    /// sets the filters that are not part of `new`
    pub fn with_filters(
        mut self,
        search: &Option<String>,
        kind: &Option<String>,
        status: &Option<String>,
        ready: bool,
        blocked: bool,
    ) -> Result<Selector, Box<dyn Error>> {
        self.search = search.as_ref().map(|s| s.to_lowercase());
        self.kind = kind.as_deref().map(Kind::parse).transpose()?;
        self.status = util::split_comma_cleanup(status.to_owned().unwrap_or_default())
            .iter()
            .map(|s| Status::parse(s))
            .collect::<Result<_, _>>()?;
        self.ready = ready;
        self.blocked = blocked;
        Ok(self)
    }

    /// check if selector is empty, except exclude_tags
    pub fn is_empty(&self) -> bool {
        return self.ids.is_empty()
            && self.children.is_empty()
            && self.parents.is_empty()
            && self.tags.is_empty()
            && self.before.is_none()
            && self.after.is_none()
            && !self.started
            && !self.stopped
            && self.search.is_none()
            && self.kind.is_none()
            && self.status.is_empty()
            && !self.ready
            && !self.blocked;
    }

    fn matches_search(&self, content: &str) -> bool {
        self.search
            .as_ref()
            .is_some_and(|s| content.to_lowercase().contains(s))
    }

    fn matches_status(&self, status: Option<Status>) -> bool {
        status.is_some_and(|s| self.status.contains(&s))
    }

    /// checks if item should be excluded
    pub fn exclude(&self, key: &String, store: &Store) -> bool {
        let item = store.get_item(key).unwrap();
        !util::contains_element(item.tags(), &self.exclude_tags)
    }

    pub fn get(&self, store: &Store, recursive: bool) -> Vec<String> {
        let mut r;
        if self.or {
            r = self.get_or(store);
        } else {
            r = self.get_and(store);
        }
        // the ItemView is not used, only the path to collect recursivly ids
        if recursive {
            let mut path1 = vec![];
            let mut path2 = vec![];
            for id in &r {
                if self.rparents {
                    let items = vec![id.to_owned()];
                    store.recursive_execute(&items, &mut path1, 0, MAX_DEPTH, true);
                }
                if self.rchildren {
                    let items = vec![id.to_owned()];
                    store.recursive_execute(&items, &mut path2, 0, MAX_DEPTH, false);
                }
            }
            for i in path1.iter().chain(path2.iter()) {
                if !r.contains(&i) {
                    r.push(i.to_string());
                }
            }
        }
        r.iter()
            .filter(|key| self.exclude(key, store))
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
    }

    fn get_or(&self, store: &Store) -> Vec<String> {
        let keys = store.get().keys().cloned().collect::<Vec<String>>();
        let keys = keys
            .iter()
            .filter(|key| {
                let item = store.get_item(&key).unwrap();
                self.is_empty()
                    || self.ids.contains(key)
                    || util::contains_element(item.children(), &self.children)
                    || util::contains_element(item.parents(), &self.parents)
                    || util::contains_element(item.tags(), &self.tags)
                    || self.before.is_some() && item.timestamp() < self.before.unwrap()
                    || self.after.is_some() && item.timestamp() > self.after.unwrap()
                    || item.is_started() && self.started
                    || item.is_stopped() && self.stopped
                    || self.matches_search(item.content())
                    || self.kind == Some(item.kind())
                    || self.matches_status(item.status())
                    || self.ready && store.is_ready(key)
                    || self.blocked && store.is_blocked(key)
            })
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        keys
    }

    fn get_and(&self, store: &Store) -> Vec<String> {
        let keys = store.get().keys().cloned().collect::<Vec<String>>();
        let keys = keys
            .iter()
            .filter(|key| {
                let item = store.get_item(&key).unwrap();
                (self.ids.is_empty() || self.ids.contains(&key))
                    && (self.children.is_empty()
                        || util::is_subset(&self.children, item.children()))
                    && (self.parents.is_empty() || util::is_subset(&self.parents, item.parents()))
                    && (self.tags.is_empty() || util::is_subset(&self.tags, item.tags()))
                    && (self.before.is_none() || item.timestamp() < self.before.unwrap())
                    && (self.after.is_none() || item.timestamp() > self.after.unwrap())
                    && (!self.started || item.is_started())
                    && (!self.stopped || item.is_stopped())
                    && (self.search.is_none() || self.matches_search(item.content()))
                    && (self.kind.is_none() || self.kind == Some(item.kind()))
                    && (self.status.is_empty() || self.matches_status(item.status()))
                    && (!self.ready || store.is_ready(key))
                    && (!self.blocked || store.is_blocked(key))
            })
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;

    fn store_with(items: Vec<Item>) -> Store {
        let mut store =
            Store::new(std::path::Path::new("/definitely/does/not/exist/cake-selector-test")).unwrap();
        for item in items {
            store.add(item).unwrap();
        }
        store
    }

    fn tagged(id: &str, tags: &[&str]) -> Item {
        let mut item = Item::new(id.to_string(), vec![], None, vec![]);
        item.append_tags(tags.iter().map(|t| t.to_string()).collect());
        item
    }

    fn selector(tags: &str, or: bool) -> Selector {
        Selector::new(
            &None,
            &None,
            &None,
            &Some(tags.to_string()),
            &None,
            &None,
            &false,
            &false,
            &0,
            &or,
        )
        .unwrap()
    }

    #[test]
    fn and_selector_requires_every_tag_to_match() {
        let store = store_with(vec![
            tagged("a", &["work", "urgent"]),
            tagged("b", &["work"]),
            tagged("c", &["urgent"]),
        ]);
        let mut result = selector("work,urgent", false).get(&store, false);
        result.sort();
        assert_eq!(result, vec!["a".to_string()]);
    }

    #[test]
    fn or_selector_matches_any_tag() {
        let store = store_with(vec![
            tagged("a", &["work"]),
            tagged("b", &["urgent"]),
            tagged("c", &["idle"]),
        ]);
        let mut result = selector("work,urgent", true).get(&store, false);
        result.sort();
        assert_eq!(result, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn exclude_tag_removes_matching_items() {
        let store = store_with(vec![
            tagged("a", &["work"]),
            tagged("b", &["work", "blocked"]),
        ]);
        let result = selector("work,~blocked", false).get(&store, false);
        assert_eq!(result, vec!["a".to_string()]);
    }

    fn filtered(search: Option<&str>, kind: Option<&str>, status: Option<&str>, ready: bool, blocked: bool) -> Selector {
        let none = Selector::new(&None, &None, &None, &None, &None, &None, &false, &false, &0, &false);
        none.unwrap()
            .with_filters(
                &search.map(String::from),
                &kind.map(String::from),
                &status.map(String::from),
                ready,
                blocked,
            )
            .unwrap()
    }

    fn sorted(mut v: Vec<String>) -> Vec<String> {
        v.sort();
        v
    }

    #[test]
    fn search_kind_and_status_filters() {
        let mut note = Item::new("n".into(), vec![], None, vec![]);
        note.set_note();
        note.set_content("# Meeting NOTES".into());
        let mut done = Item::new("d".into(), vec![], None, vec![]);
        done.set_status(Status::Done).unwrap();
        let store = store_with(vec![note, done, tagged("t", &[])]);

        assert_eq!(filtered(Some("meeting notes"), None, None, false, false).get(&store, false), vec!["n"]);
        assert_eq!(filtered(None, Some("note"), None, false, false).get(&store, false), vec!["n"]);
        assert_eq!(sorted(filtered(None, None, Some("todo,done"), false, false).get(&store, false)), vec!["d", "t"]);
        assert!(Selector::new(&None, &None, &None, &None, &None, &None, &false, &false, &0, &false)
            .unwrap()
            .with_filters(&None, &None, &Some("closed".into()), false, false)
            .is_err());
    }

    #[test]
    fn ready_and_blocked_follow_open_blockers() {
        let mut blocked = Item::new("a".into(), vec![], None, vec![]);
        blocked.set_blocked_by(vec!["b".into()]);
        let mut unblocked = Item::new("c".into(), vec![], None, vec![]);
        unblocked.set_blocked_by(vec!["d".into()]);
        let mut done = Item::new("d".into(), vec![], None, vec![]);
        done.set_status(Status::Done).unwrap();
        let store = store_with(vec![blocked, tagged("b", &[]), unblocked, done]);

        assert_eq!(filtered(None, None, None, false, true).get(&store, false), vec!["a"]);
        assert_eq!(sorted(filtered(None, None, None, true, false).get(&store, false)), vec!["b", "c"]);
    }

    #[test]
    fn is_empty_true_only_without_any_selector_criteria() {
        let none = Selector::new(
            &None, &None, &None, &None, &None, &None, &false, &false, &0, &false,
        )
        .unwrap();
        assert!(none.is_empty());

        assert!(!selector("x", false).is_empty());
    }
}
