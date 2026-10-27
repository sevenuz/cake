use super::*;
use crate::test_support::{git, init_bare_remote, Sandbox};
use serial_test::serial;
use std::fs;

fn head_subject(dir: &Path) -> String {
    super::git(dir, &["log", "-1", "--format=%s"]).unwrap()
}

fn commit_count(dir: &Path) -> usize {
    super::git(dir, &["rev-list", "--count", "HEAD"])
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
#[serial]
fn is_repo_true_from_a_subdirectory() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::create_dir("nested").unwrap();
    assert!(is_repo(&sandbox.path().join("nested")));
}

#[test]
#[serial]
fn is_repo_false_outside_a_repo() {
    let sandbox = Sandbox::new();
    assert!(!is_repo(sandbox.path()));
}

#[test]
#[serial]
fn is_store_repo_only_at_the_worktree_root() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::create_dir_all(".cake/items").unwrap();
    assert!(is_store_repo(sandbox.path()));
    assert!(!is_store_repo(&sandbox.path().join(".cake")));
}

#[test]
#[serial]
fn git_error_contains_stderr() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let err = super::git(sandbox.path(), &["checkout", "nope"]).unwrap_err();
    assert!(err.to_string().contains("nope"), "{}", err);
}

#[test]
#[serial]
fn commit_stages_given_paths_only() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::write("a.md", "a\n").unwrap();
    fs::write("b.md", "b\n").unwrap();

    commit(sandbox.path(), &[sandbox.path().join("a.md")], "cake add a").unwrap();

    assert_eq!(head_subject(sandbox.path()), "cake add a");
    let status = super::git(sandbox.path(), &["status", "--porcelain"]).unwrap();
    assert_eq!(status, "?? b.md");
}

#[test]
#[serial]
fn commit_stages_deletions() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::remove_file("README.md").unwrap();

    commit(sandbox.path(), &[sandbox.path().join("README.md")], "cake rm").unwrap();

    assert_eq!(head_subject(sandbox.path()), "cake rm");
    assert!(super::git(sandbox.path(), &["ls-files"]).unwrap().is_empty());
}

#[test]
#[serial]
fn commit_is_a_noop_without_changes() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    commit(sandbox.path(), &[sandbox.path().join("README.md")], "nothing").unwrap();
    commit(sandbox.path(), &[], "nothing").unwrap();
    assert_eq!(commit_count(sandbox.path()), 1);
}

#[test]
#[serial]
fn commit_works_from_another_cwd() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::write("a.md", "a\n").unwrap();
    fs::create_dir("elsewhere").unwrap();
    std::env::set_current_dir("elsewhere").unwrap();

    commit(sandbox.path(), &[sandbox.path().join("a.md")], "cake add a").unwrap();
    assert_eq!(commit_count(sandbox.path()), 2);
    git("status", &[]); // cwd is still inside the repo, untouched
}

fn write_and_commit(dir: &Path, file: &str, content: &str) {
    fs::write(dir.join(file), content).unwrap();
    commit(dir, &[dir.join(file)], &format!("write {}", file)).unwrap();
}

#[test]
#[serial]
fn add_worktree_creates_orphan_branch_without_touching_main() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let store = sandbox.path().join(".cake");

    assert!(!add_worktree(sandbox.path(), &store, "cake/main", "origin").unwrap());
    exclude(sandbox.path(), "/.cake/").unwrap();
    write_and_commit(&store, "a.md", "a\n");

    assert_eq!(super::git(&store, &["branch", "--show-current"]).unwrap(), "cake/main");
    assert_eq!(commit_count(&store), 1, "orphan: no code history");
    assert_eq!(super::git(sandbox.path(), &["branch", "--show-current"]).unwrap(), "main");
    assert!(super::git(sandbox.path(), &["status", "--porcelain"]).unwrap().is_empty());
    assert!(is_store_repo(&store));
}

#[test]
#[serial]
fn add_worktree_reattaches_existing_local_branch() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let store = sandbox.path().join(".cake");
    add_worktree(sandbox.path(), &store, "cake/main", "origin").unwrap();
    write_and_commit(&store, "a.md", "a\n");
    fs::remove_dir_all(&store).unwrap();

    assert!(add_worktree(sandbox.path(), &store, "cake/main", "origin").unwrap());
    assert_eq!(fs::read_to_string(store.join("a.md")).unwrap(), "a\n");
}

#[test]
#[serial]
fn add_worktree_attaches_branch_from_remote_with_upstream() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote = sandbox.path().join("remote.git");
    init_bare_remote(&remote);
    git("remote", &["add", "origin", remote.to_str().unwrap()]);
    git("push", &["-q", "origin", "main"]);
    let store = sandbox.path().join(".cake");
    add_worktree(sandbox.path(), &store, "cake/main", "origin").unwrap();
    write_and_commit(&store, "a.md", "a\n");
    super::git(&store, &["push", "-q", "origin", "cake/main"]).unwrap();

    let clone = sandbox.path().join("clone");
    git("clone", &["-q", remote.to_str().unwrap(), clone.to_str().unwrap()]);
    let store2 = clone.join(".cake");
    assert!(add_worktree(&clone, &store2, "cake/main", "origin").unwrap());

    assert_eq!(fs::read_to_string(store2.join("a.md")).unwrap(), "a\n");
    assert_eq!(
        super::git(&store2, &["rev-parse", "--abbrev-ref", "@{u}"]).unwrap(),
        "origin/cake/main"
    );
}

#[test]
#[serial]
fn exclude_appends_once() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    exclude(sandbox.path(), "/.cake/").unwrap();
    exclude(sandbox.path(), "/.cake/").unwrap();
    let content = fs::read_to_string(".git/info/exclude").unwrap();
    assert_eq!(content.matches("/.cake/").count(), 1);
}
