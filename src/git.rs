use std::{
    error::Error,
    process::{Command, Output},
};

use crate::config::Config;

#[cfg(test)]
mod tests;

pub fn is_repo() -> bool {
    Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_if_branch_exists(config: &Config) -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(
        Command::new("git")
            .arg("branch")
            .arg("--list")
            .arg(&config.git_branch_name)
            .output()?
            .stdout,
    )?;
    Ok(!output.trim().is_empty())
}

pub fn check_if_remote_branch_exists(config: &Config) -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(
        Command::new("git")
            .arg("branch")
            .arg("--all")
            .arg("--list")
            .arg(format!(
                "{}/{}",
                &config.git_remote_name, &config.git_branch_name
            ))
            .output()?
            .stdout,
    )?;
    Ok(!output.trim().is_empty())
}

pub fn check_conflicts() -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(
        Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .output()?
            .stdout,
    )?;
    // an unmerged (conflicted) path is reported with a `U` in either the
    // staged or unstaged status column, e.g. "UU", "AU", "UD"
    Ok(output
        .lines()
        .any(|line| line.get(0..2).is_some_and(|xy| xy.contains('U'))))
}

pub fn stash() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").output()?)
}

pub fn stash_pop() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").arg("pop").output()?)
}

/// Stashes uncommitted changes on creation; checks `original_branch` back
/// out and restores the stash on drop, including on early return via `?`.
pub struct StashGuard {
    original_branch: String,
}

impl StashGuard {
    pub fn new(original_branch: &str) -> Result<Self, Box<dyn Error>> {
        stash()?;
        Ok(Self {
            original_branch: original_branch.to_string(),
        })
    }
}

impl Drop for StashGuard {
    fn drop(&mut self) {
        let _ = checkout_branch(&self.original_branch);
        let _ = stash_pop();
    }
}

pub fn create_branch(config: &Config) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("branch")
        .arg(&config.git_branch_name)
        .output()?)
}

pub fn current_branch_name() -> Result<String, Box<dyn Error>> {
    let mut s = String::from_utf8(
        Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()?
            .stdout,
    )?;
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    Ok(s)
}

pub fn checkout_branch(branch: &str) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("checkout").arg(branch).output()?)
}

pub fn add(config: &Config) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("add")
        .arg(&config.save_file_name)
        .output()?)
}

pub fn commit(message: &str) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .output()?)
}

pub fn fetch(config: &Config) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("fetch")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .output()?)
}

pub fn rebase() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("rebase").output()?)
}

pub fn rebase_abort() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("rebase").arg("--abort").output()?)
}

pub fn push(config: &Config) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("push")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .output()?)
}

pub fn push_set_upstream(config: &Config) -> Result<Output, Box<dyn Error>> {
    //git push --set-upstream origin someti
    Ok(Command::new("git")
        .arg("push")
        .arg("--set-upstream")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .output()?)
}
