//! the known stores, ~/.config/cake/stores.json: name -> store dir and the
//! project dirs that use it without a .cake inside (e.g. a paper repo).
//! Written by cake (init, first use), may be edited by hand.

use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub store: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirs: Vec<PathBuf>,
}

pub type Stores = BTreeMap<String, Entry>;

fn file() -> Result<PathBuf, Box<dyn Error>> {
    Ok(util::config_path()?.join("stores.json"))
}

/// `~/x` -> home/x
fn expand(p: &Path) -> PathBuf {
    match (p.strip_prefix("~"), std::env::var_os("HOME")) {
        (Ok(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => p.to_path_buf(),
    }
}

fn absolute(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// the list, empty if there is none; a broken file is an error
pub fn load() -> Result<Stores, Box<dyn Error>> {
    let Ok(text) = std::fs::read_to_string(file()?) else {
        return Ok(Stores::new());
    };
    let mut stores: Stores =
        serde_json::from_str(&text).map_err(|e| format!("{}: {}", file().unwrap().display(), e))?;
    for e in stores.values_mut() {
        e.store = expand(&e.store);
        e.dirs = e.dirs.iter().map(|d| expand(d)).collect();
    }
    Ok(stores)
}

fn save(stores: &Stores) -> Result<(), Box<dyn Error>> {
    std::fs::write(file()?, serde_json::to_string_pretty(stores)? + "\n")?;
    Ok(())
}

/// adds `store` under `name` (or name-2, ..), adds `dirs` if it is known
pub fn register(store: &Path, name: &str, dirs: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let mut stores = load()?;
    let store = absolute(store);
    let dirs: Vec<PathBuf> = dirs.iter().map(|d| absolute(d)).collect();
    if let Some(e) = stores.values_mut().find(|e| absolute(&e.store) == store) {
        let new: Vec<PathBuf> = dirs.into_iter().filter(|d| !e.dirs.contains(d)).collect();
        if new.is_empty() {
            return Ok(());
        }
        e.dirs.extend(new);
    } else {
        let name = (1..)
            .map(|n| if n == 1 { name.to_string() } else { format!("{}-{}", name, n) })
            .find(|n| !stores.contains_key(n))
            .unwrap();
        stores.insert(name, Entry { store, dirs });
    }
    save(&stores)
}

/// the store whose `dirs` hold `path`, the deepest dir wins
pub fn for_dir(stores: &Stores, path: &Path) -> Option<PathBuf> {
    let path = absolute(path);
    stores
        .values()
        .flat_map(|e| e.dirs.iter().map(move |d| (d, &e.store)))
        .filter(|(d, _)| path.starts_with(d))
        .max_by_key(|(d, _)| d.components().count())
        .map(|(_, s)| s.clone())
}

/// a name from the list, `global`, or a path
pub fn resolve(stores: &Stores, target: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(e) = stores.get(target) {
        return Ok(e.store.clone());
    }
    if target == "global" || target == "-g" {
        return util::global_store();
    }
    Ok(expand(Path::new(target)))
}

/// (name, store) of every known store that exists, the global one last
pub fn all() -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
    let mut out: Vec<(String, PathBuf)> = load()?
        .into_iter()
        .filter(|(_, e)| e.store.join("items").is_dir())
        .map(|(n, e)| (n, e.store))
        .collect();
    let global = util::global_store()?;
    if global.join("items").is_dir() && !out.iter().any(|(_, s)| absolute(s) == absolute(&global)) {
        out.push(("global".to_string(), global));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Sandbox;
    use serial_test::serial;

    #[test]
    #[serial]
    fn register_names_uniquely_and_merges_dirs() {
        let sandbox = Sandbox::new();
        let (a, b, d) = (sandbox.path().join("a"), sandbox.path().join("b"), sandbox.path().join("d"));
        for p in [&a, &b, &d] {
            std::fs::create_dir_all(p).unwrap();
        }
        register(&a, "paper", &[]).unwrap();
        register(&b, "paper", &[]).unwrap();
        register(&a, "other", &[d.clone()]).unwrap();
        let stores = load().unwrap();
        assert_eq!(stores.keys().collect::<Vec<_>>(), vec!["paper", "paper-2"]);
        assert_eq!(stores["paper"].dirs, vec![absolute(&d)]);
        assert_eq!(for_dir(&stores, &d.join("sub")), Some(absolute(&a)));
        assert_eq!(for_dir(&stores, &b), None);
        assert_eq!(resolve(&stores, "paper-2").unwrap(), absolute(&b));
    }
}
