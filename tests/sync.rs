//! End-to-end: runs the cake binary against a bare remote and two clones.

mod common;

use common::*;
use std::fs;
use std::path::Path;

fn items(repo: &Path) -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(repo.join(".cake/items"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    ids.sort();
    ids
}

fn assert_code_untouched(repo: &Path) {
    assert_eq!(git(repo, &["branch", "--show-current"]), "main");
    assert_eq!(git(repo, &["status", "--porcelain"]), "");
    assert_eq!(git(repo, &["log", "--format=%s"]), "code");
}

#[test]
fn two_devices_sync_through_the_remote() {
    let env = Env::new();
    let a = env.repo("a");

    env.cake_ok(&a, &["init"]);
    env.cake_ok(&a, &["add", "one", "-m", "# one"]);
    assert!(env.cake_err(&a, &["sync"]).contains("--publish"));
    env.cake_ok(&a, &["sync", "--publish"]);

    let b = env.repo("b");
    assert!(env.cake_ok(&b, &["init"]).contains("Attached"));
    assert_eq!(items(&b), vec!["one.md"]);

    // commands from a subdirectory commit into the store
    fs::create_dir(b.join("src")).unwrap();
    env.cake_ok(&b.join("src"), &["add", "two", "-m", "# two"]);
    assert_eq!(git(&b.join(".cake"), &["log", "-1", "--format=%s"]), "cake add two -m # two");
    env.cake_ok(&b, &["sync"]);
    env.cake_ok(&a, &["sync"]);
    assert_eq!(items(&a), vec!["one.md", "two.md"]);

    assert_code_untouched(&a);
    assert_code_untouched(&b);
}

#[test]
fn conflict_blocks_writes_until_resolved() {
    let env = Env::new();
    let a = env.repo("a");
    env.cake_ok(&a, &["init"]);
    env.cake_ok(&a, &["add", "one", "-m", "# one"]);
    env.cake_ok(&a, &["sync", "--publish"]);
    let b = env.repo("b");
    env.cake_ok(&b, &["init"]);

    env.cake_ok(&a, &["tag", "one", "from-a"]);
    env.cake_ok(&a, &["sync"]);
    env.cake_ok(&b, &["tag", "one", "from-b"]);

    assert!(env.cake_err(&b, &["sync"]).contains("items/one.md"));
    assert!(env.cake_err(&b, &["add", "-m", "# new"]).contains("items/one.md"));
    env.cake_ok(&b, &["ls"]); // reads still work
    assert!(env.cake_err(&b, &["sync", "--continue"]).contains("markers"));

    let item = b.join(".cake/items/one.md");
    let resolved: String = fs::read_to_string(&item)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("<<<<<<<") && !l.starts_with(">>>>>>>") && *l != "=======")
        .filter(|l| !l.starts_with("tags: [from-a]"))
        .map(|l| format!("{}\n", l.replace("tags: [from-b]", "tags: [from-a, from-b]")))
        .collect();
    fs::write(&item, resolved).unwrap();
    env.cake_ok(&b, &["sync", "--continue"]);
    env.cake_ok(&b, &["add", "-m", "# new"]);

    env.cake_ok(&a, &["sync"]);
    assert!(fs::read_to_string(a.join(".cake/items/one.md"))
        .unwrap()
        .contains("tags: [from-a, from-b]"));
    assert_code_untouched(&a);
    assert_code_untouched(&b);
}

#[test]
fn global_store_is_its_own_repo() {
    let env = Env::new();
    let a = env.repo("a");
    env.cake_ok(&a, &["init"]);

    // used before init: plain dir, adopted by init
    env.cake_ok(&a, &["-g", "add", "early", "-m", "# early"]);
    env.cake_ok(&a, &["-g", "init"]);
    assert!(env.cake_err(&a, &["-g", "init"]).contains("already"));
    env.cake_ok(&a, &["-g", "add", "note", "-m", "# personal"]);

    let global = env.path("data/cake");
    assert!(global.join("items/note.md").exists());
    assert!(!a.join(".cake/items/note.md").exists());
    assert_eq!(git(&global, &["log", "--format=%s"]), "cake -g add note -m # personal\ncake: init");
    assert_eq!(git(&global, &["ls-files"]), ".gitignore\nitems/early.md\nitems/note.md");
    assert_eq!(git(&global, &["branch", "--show-current"]), "main");
}

#[test]
fn plain_store_outside_git_does_not_commit() {
    let env = Env::new();
    let dir = env.path("plain");
    fs::create_dir(&dir).unwrap();
    env.cake_ok(&dir, &["init"]);
    env.cake_ok(&dir, &["add", "x", "-m", "# x"]);
    assert!(dir.join(".cake/items/x.md").exists());
    assert!(!dir.join(".cake/.git").exists());
    assert!(env.cake_err(&dir, &["sync"]).contains("not a git store"));
}
