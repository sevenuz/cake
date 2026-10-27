use super::*;
use crate::test_support::{git, Sandbox};
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
