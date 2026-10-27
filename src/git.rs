use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[cfg(test)]
mod tests;

fn run(dir: &Path, args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("git").arg("-C").arg(dir).args(args).output()?)
}

/// runs git in `dir`, returns trimmed stdout or an error with git's stderr
fn git(dir: &Path, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let out = run(dir, args)?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

/// runs git in `dir`, true on exit code 0, false otherwise
fn git_ok(dir: &Path, args: &[&str]) -> Result<bool, Box<dyn Error>> {
    Ok(run(dir, args)?.status.success())
}

pub fn is_repo(dir: &Path) -> bool {
    git_ok(dir, &["rev-parse", "--is-inside-work-tree"]).unwrap_or(false)
}

/// root of the working tree containing `dir`
pub fn toplevel(dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(git(dir, &["rev-parse", "--show-toplevel"])?))
}

/// true if `dir` is the root of a git working tree (main or linked worktree),
/// i.e. a store whose changes get committed. A plain `.cake` dir inside a code
/// repo is not, so cake never commits to the code branch.
pub fn is_store_repo(dir: &Path) -> bool {
    match (toplevel(dir), dir.canonicalize()) {
        (Ok(top), Ok(dir)) => top.canonicalize().map_or(false, |t| t == dir),
        _ => false,
    }
}

/// stages `paths` (also deletions) and commits them, no-op if nothing changed
pub fn commit(dir: &Path, paths: &[PathBuf], message: &str) -> Result<(), Box<dyn Error>> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["add", "-A", "--"];
    args.extend(paths.iter().map(|p| p.to_str().unwrap()));
    git(dir, &args)?;
    if git_ok(dir, &["diff", "--cached", "--quiet"])? {
        return Ok(());
    }
    git(dir, &["commit", "-q", "-m", message])?;
    Ok(())
}
