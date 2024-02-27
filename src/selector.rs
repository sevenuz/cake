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
        })
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
            && !self.stopped;
    }

    pub fn get(&self, store: &Store, recursive: bool) -> Vec<String> {
        let mut r;
        if self.or {
            r = self.get_or(store);
        } else {
            r = self.get_and(store);
        }
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
        r
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
            })
            .filter(|key| {
                let item = store.get_item(key).unwrap();
                !util::contains_element(item.tags(), &self.exclude_tags)
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
            })
            .filter(|key| {
                let item = store.get_item(key).unwrap();
                !util::contains_element(item.tags(), &self.exclude_tags)
            })
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        keys
    }
}
