use super::*;
use crate::test_support::{default_config, git, init_bare_remote, Sandbox};
use serial_test::serial;
use std::fs;
use std::process::Command as StdCommand;

#[test]
#[serial]
fn is_repo_true_at_repo_root() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    assert!(is_repo());
}

#[test]
#[serial]
fn is_repo_false_outside_a_repo() {
    let _sandbox = Sandbox::new(); // fresh tempdir, no `git init`
    assert!(!is_repo());
}

#[test]
#[serial]
fn is_repo_detects_from_a_subdirectory() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::create_dir("nested").unwrap();
    std::env::set_current_dir("nested").unwrap();
    // BUG (see todo.md): is_repo() only checks `.git` in the current
    // directory, unlike git itself, which resolves repos from subdirectories.
    assert!(is_repo());
}

#[test]
#[serial]
fn current_branch_name_returns_active_branch() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    assert_eq!(current_branch_name().unwrap(), "main");
}

#[test]
#[serial]
fn checkout_branch_switches_active_branch() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("branch", &["feature"]);
    checkout_branch("feature").unwrap();
    assert_eq!(current_branch_name().unwrap(), "feature");
}

#[test]
#[serial]
fn stash_then_pop_restores_dirty_working_tree() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    fs::write("README.md", "changed\n").unwrap();

    stash().unwrap();
    assert_eq!(fs::read_to_string("README.md").unwrap(), "init\n");

    stash_pop().unwrap();
    assert_eq!(fs::read_to_string("README.md").unwrap(), "changed\n");
}

#[test]
#[serial]
fn check_if_branch_exists_true_for_exact_match() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("branch", &["cake"]);
    assert!(check_if_branch_exists(&default_config()).unwrap());
}

#[test]
#[serial]
fn check_if_branch_exists_false_when_only_similarly_named_branch_exists() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("branch", &["cake-experiment"]);
    // BUG (see todo.md): check_if_branch_exists does a substring match on
    // `git branch` output, so "cake-experiment" is currently mistaken for "cake".
    assert!(!check_if_branch_exists(&default_config()).unwrap());
}

#[test]
#[serial]
fn check_if_remote_branch_exists_true_after_push() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote_path = sandbox.path().join("remote.git");
    init_bare_remote(&remote_path);
    git("remote", &["add", "origin", remote_path.to_str().unwrap()]);
    git("branch", &["cake"]);
    git("push", &["origin", "cake"]);

    assert!(check_if_remote_branch_exists(&default_config()).unwrap());
}

#[test]
#[serial]
fn check_if_remote_branch_exists_false_when_not_pushed() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote_path = sandbox.path().join("remote.git");
    init_bare_remote(&remote_path);
    git("remote", &["add", "origin", remote_path.to_str().unwrap()]);
    git("branch", &["cake"]);

    assert!(!check_if_remote_branch_exists(&default_config()).unwrap());
}

#[test]
#[serial]
fn check_if_remote_branch_exists_false_when_only_similarly_named_remote_branch_exists() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote_path = sandbox.path().join("remote.git");
    init_bare_remote(&remote_path);
    git("remote", &["add", "origin", remote_path.to_str().unwrap()]);
    git("branch", &["cake-experiment"]);
    git("push", &["origin", "cake-experiment"]);

    // Same substring-match bug as above, applied to the remote-branch check.
    assert!(!check_if_remote_branch_exists(&default_config()).unwrap());
}

#[test]
#[serial]
fn check_conflicts_detects_rebase_conflict() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("checkout", &["-b", "feature"]);
    fs::write("README.md", "feature change\n").unwrap();
    git("commit", &["-am", "feature change"]);
    git("checkout", &["main"]);
    fs::write("README.md", "main change\n").unwrap();
    git("commit", &["-am", "main change"]);
    git("checkout", &["feature"]);

    // Expected to fail with a conflict; ignore the exit code here.
    let _ = StdCommand::new("git").args(["rebase", "main"]).output();

    assert!(check_conflicts().unwrap());
    let _ = StdCommand::new("git").args(["rebase", "--abort"]).output();
}

#[test]
#[serial]
fn check_conflicts_false_when_rebase_is_clean() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("checkout", &["-b", "feature"]);
    fs::write("extra.md", "new file\n").unwrap();
    git("add", &["extra.md"]);
    git("commit", &["-q", "-m", "add extra file"]);

    let status = StdCommand::new("git")
        .args(["rebase", "main"])
        .status()
        .unwrap();
    assert!(status.success());

    assert!(!check_conflicts().unwrap());
}

#[test]
#[serial]
fn stash_guard_restores_branch_and_stash_on_drop() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("branch", &["cake"]);
    fs::write("README.md", "dirty\n").unwrap();

    {
        let _guard = StashGuard::new("main").unwrap();
        checkout_branch("cake").unwrap();
        assert_eq!(current_branch_name().unwrap(), "cake");
        assert_eq!(fs::read_to_string("README.md").unwrap(), "init\n");
    }

    assert_eq!(current_branch_name().unwrap(), "main");
    assert_eq!(fs::read_to_string("README.md").unwrap(), "dirty\n");
}

#[test]
#[serial]
fn stash_guard_restores_on_early_return_via_question_mark() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    git("branch", &["cake"]);
    fs::write("README.md", "dirty\n").unwrap();

    fn do_work_that_fails() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = StashGuard::new("main")?;
        checkout_branch("cake")?;
        Err("simulated failure after entering the guarded branch".into())
    }

    assert!(do_work_that_fails().is_err());
    assert_eq!(current_branch_name().unwrap(), "main");
    assert_eq!(fs::read_to_string("README.md").unwrap(), "dirty\n");
}

#[test]
#[serial]
fn fetch_and_rebase_apply_remote_changes_cleanly() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote_path = sandbox.path().join("remote.git");
    init_bare_remote(&remote_path);
    git("remote", &["add", "origin", remote_path.to_str().unwrap()]);

    let mut config = default_config();
    config.git_branch_name = "main".to_string();
    config.git_remote_name = "origin".to_string();
    // `git rebase` (no args, as called by git::rebase()) rebases onto the
    // current branch's upstream, so that has to be established first - same
    // as what `cake init --git --remote` does via push_set_upstream.
    push_set_upstream(&config).unwrap();

    // simulate someone else pushing a commit to origin/main via a second clone
    let clone_path = sandbox.path().join("clone");
    let status = StdCommand::new("git")
        .args([
            "clone",
            "-q",
            remote_path.to_str().unwrap(),
            clone_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    fs::write(clone_path.join("README.md"), "from clone\n").unwrap();
    StdCommand::new("git")
        .current_dir(&clone_path)
        .args(["commit", "-am", "clone change"])
        .status()
        .unwrap();
    StdCommand::new("git")
        .current_dir(&clone_path)
        .args(["push", "origin", "main"])
        .status()
        .unwrap();

    fetch(&config).unwrap();
    rebase().unwrap();

    assert!(!check_conflicts().unwrap());
    assert_eq!(fs::read_to_string("README.md").unwrap(), "from clone\n");
}

#[test]
#[serial]
fn push_uploads_local_commits_to_remote() {
    let sandbox = Sandbox::new();
    sandbox.init_git_repo();
    let remote_path = sandbox.path().join("remote.git");
    init_bare_remote(&remote_path);
    git("remote", &["add", "origin", remote_path.to_str().unwrap()]);

    let mut config = default_config();
    config.git_branch_name = "main".to_string();
    config.git_remote_name = "origin".to_string();

    push_set_upstream(&config).unwrap();
    assert!(check_if_remote_branch_exists(&config).unwrap());

    fs::write("README.md", "pushed change\n").unwrap();
    git("commit", &["-am", "pushed change"]);
    push(&config).unwrap();

    let out = StdCommand::new("git")
        .args(["log", "-1", "--format=%s", "refs/remotes/origin/main"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "pushed change"
    );
}
