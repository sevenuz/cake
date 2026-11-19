//! the known stores, ~/.config/cake/stores.json: name -> store (a .cake)
//! and the dirs that use it without a .cake of their own (e.g. a paper repo).
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
    pub dirs: Vec<Dir>,
}

/// a dir that uses the store, written as a plain path or, with a default
/// goal for what is captured there, as {"path", "parent"}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Dir {
    Path(PathBuf),
    Linked { path: PathBuf, parent: String },
}

impl Dir {
    pub fn path(&self) -> &Path {
        match self {
            Dir::Path(p) | Dir::Linked { path: p, .. } => p,
        }
    }

    pub fn parent(&self) -> Option<&str> {
        match self {
            Dir::Path(_) => None,
            Dir::Linked { parent, .. } => Some(parent),
        }
    }

    fn new(path: PathBuf, parent: Option<String>) -> Dir {
        match parent {
            Some(parent) => Dir::Linked { path, parent },
            None => Dir::Path(path),
        }
    }
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
        e.store = util::store_dir(&expand(&e.store));
        e.dirs = e.dirs.iter().map(|d| Dir::new(expand(d.path()), d.parent().map(String::from))).collect();
    }
    Ok(stores)
}

/// atomic: parallel cake runs read either the old or the new list, never
/// a half written one
fn save(stores: &Stores) -> Result<(), Box<dyn Error>> {
    let path = file()?;
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string_pretty(stores)? + "\n")?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// adds `store` under `name` (or name-2, ..), adds `dirs` if it is known
pub fn register(store: &Path, name: &str, dirs: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let mut stores = load()?;
    let store = absolute(store);
    let dirs: Vec<PathBuf> = dirs.iter().map(|d| absolute(d)).collect();
    if let Some(e) = stores.values_mut().find(|e| absolute(&e.store) == store) {
        let new: Vec<Dir> = dirs.into_iter().filter(|d| !e.dirs.iter().any(|x| x.path() == d)).map(Dir::Path).collect();
        if new.is_empty() {
            return Ok(());
        }
        e.dirs.extend(new);
    } else {
        let name = (1..)
            .map(|n| if n == 1 { name.to_string() } else { format!("{}-{}", name, n) })
            .find(|n| !stores.contains_key(n))
            .unwrap();
        stores.insert(name, Entry { store, dirs: dirs.into_iter().map(Dir::Path).collect() });
    }
    save(&stores)
}

/// `dir` uses `store` (registered as `name` if new), captures there go
/// under `parent`; linking a dir again replaces its parent. Another store
/// linking the same dir is an error: the dir would be ambiguous.
pub fn link(store: &Path, name: &str, dir: &Path, parent: Option<&str>) -> Result<(), Box<dyn Error>> {
    let (store, dir) = (absolute(store), absolute(dir));
    let stores = load()?;
    if let Some((other, _)) = stores.iter().find(|(_, e)| absolute(&e.store) != store && e.dirs.iter().any(|d| d.path() == dir)) {
        return Err(crate::error::invalid(format!("{} is already linked to the store '{}'.", dir.display(), other)));
    }
    register(&store, name, std::slice::from_ref(&dir))?;
    let mut stores = load()?;
    let e = stores.values_mut().find(|e| absolute(&e.store) == store).expect("registered");
    for d in e.dirs.iter_mut().filter(|d| d.path() == dir) {
        *d = Dir::new(dir.clone(), parent.map(String::from));
    }
    save(&stores)
}

/// a goal of `store` got the id `new`: links with `old` follow
pub fn rename_parent(store: &Path, old: &str, new: &str) -> Result<(), Box<dyn Error>> {
    let store = absolute(store);
    let mut stores = load()?;
    let mut changed = false;
    for e in stores.values_mut().filter(|e| absolute(&e.store) == store) {
        for d in e.dirs.iter_mut().filter(|d| d.parent() == Some(old)) {
            *d = Dir::new(d.path().to_path_buf(), Some(new.to_string()));
            changed = true;
        }
    }
    if changed {
        save(&stores)?;
    }
    Ok(())
}

/// the store whose `dirs` hold `path`, the deepest dir wins
pub fn for_dir(stores: &Stores, path: &Path) -> Option<PathBuf> {
    link_of(stores, path).map(|(s, _)| s)
}

/// (store, default goal) of the deepest linked dir holding `path`
pub fn link_of(stores: &Stores, path: &Path) -> Option<(PathBuf, Option<String>)> {
    let path = absolute(path);
    stores
        .values()
        .flat_map(|e| e.dirs.iter().map(move |d| (d, &e.store)))
        .filter(|(d, _)| path.starts_with(d.path()))
        .max_by_key(|(d, _)| d.path().components().count())
        .map(|(d, s)| (s.clone(), d.parent().map(String::from)))
}

/// a name from the list, `global`, or a path (a dir or its .cake)
pub fn resolve(stores: &Stores, target: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(e) = stores.get(target) {
        return Ok(e.store.clone());
    }
    if target == "global" || target == "-g" {
        return util::global_store();
    }
    Ok(util::store_dir(&expand(Path::new(target))))
}

/// (name, store) of every known store that exists, the global one last
pub fn all() -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
    let stores = load()?;
    for e in stores.values() {
        util::upgrade_layout(&e.store)?;
    }
    let global = util::global_store()?;
    util::upgrade_layout(&global)?;
    let mut out: Vec<(String, PathBuf)> = stores
        .into_iter()
        .filter(|(_, e)| e.store.join("items").is_dir())
        .map(|(n, e)| (n, e.store))
        .collect();
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
    fn links_carry_a_default_goal_that_follows_renames() {
        let sandbox = Sandbox::new();
        let (s, t, g, p) = (sandbox.path().join("s/.cake"), sandbox.path().join("t/.cake"), sandbox.path().join("g"), sandbox.path().join("p"));
        for x in [&s, &t, &g, &p, &g.join("sub")] {
            std::fs::create_dir_all(x).unwrap();
        }
        link(&s, "phd", &g, Some("gekkofs")).unwrap();
        link(&s, "phd", &p, None).unwrap();
        let stores = load().unwrap();
        assert_eq!(link_of(&stores, &g.join("sub")), Some((absolute(&s), Some("gekkofs".into()))));
        assert_eq!(link_of(&stores, &p), Some((absolute(&s), None)));
        let text = std::fs::read_to_string(file().unwrap()).unwrap();
        assert!(text.contains("\"parent\": \"gekkofs\""), "{}", text);
        // an old plain list still loads
        assert!(text.contains(&format!("\"{}\"", absolute(&p).display())), "{}", text);
        rename_parent(&s, "gekkofs", "gfs").unwrap();
        assert_eq!(link_of(&load().unwrap(), &g).unwrap().1.as_deref(), Some("gfs"));
        link(&s, "phd", &g, None).unwrap();
        assert_eq!(link_of(&load().unwrap(), &g).unwrap().1, None);
        assert!(link(&t, "other", &g, None).is_err());
    }

    #[test]
    #[serial]
    fn register_names_uniquely_and_merges_dirs() {
        let sandbox = Sandbox::new();
        let (a, b, d) = (sandbox.path().join("a/.cake"), sandbox.path().join("b/.cake"), sandbox.path().join("d"));
        for p in [&a, &b, &d] {
            std::fs::create_dir_all(p).unwrap();
        }
        register(&a, "paper", &[]).unwrap();
        register(&b, "paper", &[]).unwrap();
        register(&a, "other", &[d.clone()]).unwrap();
        let stores = load().unwrap();
        assert_eq!(stores.keys().collect::<Vec<_>>(), vec!["paper", "paper-2"]);
        assert_eq!(stores["paper"].dirs, vec![Dir::Path(absolute(&d))]);
        assert_eq!(for_dir(&stores, &d.join("sub")), Some(absolute(&a)));
        assert_eq!(for_dir(&stores, &b), None);
        assert_eq!(resolve(&stores, "paper-2").unwrap(), absolute(&b));
        assert_eq!(resolve(&stores, a.parent().unwrap().to_str().unwrap()).unwrap(), a, "a dir means its .cake");
    }
}
