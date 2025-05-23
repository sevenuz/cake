use std::{
    error::Error,
    path::Path,
    process::{Command, Output},
};

use crate::config::Config;

pub fn is_repo() -> bool {
    Path::new(".git").exists()
}

pub fn check_if_branch_exists(config: &Config) -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(Command::new("git").arg("branch").output()?.stdout)?;
    Ok(output.find(&config.git_branch_name).is_some())
}

pub fn check_if_remote_branch_exists(config: &Config) -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(
        Command::new("git")
            .arg("branch")
            .arg("--all")
            .output()?
            .stdout,
    )?;
    Ok(output
        .find(format!(
            "remotes/{}/{}",
            &config.git_remote_name, &config.git_branch_name
        ).as_str())
        .is_some())
}

pub fn check_conflicts() -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(
        Command::new("git")
            .env("LANG", "en_US")
            .arg("status")
            .output()?
            .stdout,
    )?;
    Ok(output.find("CONFLICT").is_some())
}

pub fn stash() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").output()?)
}

pub fn stash_pop() -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").arg("pop").output()?)
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
