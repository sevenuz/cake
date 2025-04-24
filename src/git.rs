use std::{
    error::Error, path::Path, process::{Command, ExitStatus}
};

use crate::config::Config;

pub fn is_repo() -> bool {
    Path::new(".git").exists()
}

pub fn check_if_branch_exists(config: &Config) -> Result<bool, Box<dyn Error>> {
    let output = String::from_utf8(Command::new("git").arg("branch").output()?.stdout)?;
    Ok(output.find(&config.git_branch_name).is_some())
}

pub fn stash() -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").status()?)
}

pub fn stash_pop() -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git").arg("stash").arg("pop").status()?)
}

pub fn create_branch(config: &Config) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("branch")
        .arg(&config.git_branch_name)
        .status()?)
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

pub fn checkout_branch(branch: &str) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git").arg("checkout").arg(branch).status()?)
}

pub fn add(config: &Config) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("add")
        .arg(&config.save_file_name)
        .status()?)
}

pub fn commit(message: &str) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("commit")
        .arg("-m\"".to_owned() + message + "\"")
        .status()?)
}

pub fn fetch(config: &Config) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("fetch")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .status()?)
}

pub fn rebase() -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("rebase")
        .status()?)
}

pub fn push(config: &Config) -> Result<ExitStatus, Box<dyn Error>> {
    Ok(Command::new("git")
        .arg("push")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .status()?)
}

pub fn push_set_upstream(config: &Config) -> Result<ExitStatus, Box<dyn Error>> {
    //git push --set-upstream origin someti
    Ok(Command::new("git")
        .arg("push")
        .arg("--set-upstream")
        .arg(&config.git_remote_name)
        .arg(&config.git_branch_name)
        .status()?)
}
