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

fn branch_exists(dir: &Path, branch: &str) -> Result<bool, Box<dyn Error>> {
    git_ok(dir, &["rev-parse", "-q", "--verify", &format!("refs/heads/{}", branch)])
}

fn has_remote(dir: &Path, remote: &str) -> Result<bool, Box<dyn Error>> {
    git_ok(dir, &["remote", "get-url", remote])
}

/// asks the remote whether `branch` exists (network), errors on other failures
fn remote_branch_exists(dir: &Path, remote: &str, branch: &str) -> Result<bool, Box<dyn Error>> {
    let out = run(dir, &["ls-remote", "--exit-code", "--heads", remote, branch])?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(2) => Ok(false),
        _ => Err(format!(
            "git ls-remote {} failed: {}",
            remote,
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into()),
    }
}

/// Attaches a worktree for `branch` at `path`. Uses the local branch if it
/// exists, else the one on `remote`, else creates an orphan branch.
/// Returns false if the branch was newly created (worktree is empty).
pub fn add_worktree(
    repo: &Path,
    path: &Path,
    branch: &str,
    remote: &str,
) -> Result<bool, Box<dyn Error>> {
    let path = path.to_str().unwrap();
    git(repo, &["worktree", "prune"])?;
    if branch_exists(repo, branch)? {
        git(repo, &["worktree", "add", path, branch])?;
        return Ok(true);
    }
    if has_remote(repo, remote)? && remote_branch_exists(repo, remote, branch)? {
        let tracking = format!("{}/{}", remote, branch);
        let refspec = format!("{}:refs/remotes/{}", branch, tracking);
        git(repo, &["fetch", "-q", remote, &refspec])?;
        git(repo, &["worktree", "add", "-q", "--track", "-b", branch, path, &tracking])?;
        return Ok(true);
    }
    git(repo, &["worktree", "add", "-q", "--orphan", "-b", branch, path])?;
    Ok(false)
}

/// adds `pattern` to the repo's local .git/info/exclude (shared by all worktrees)
pub fn exclude(repo: &Path, pattern: &str) -> Result<(), Box<dyn Error>> {
    let common = git(repo, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    let info = Path::new(&common).join("info");
    std::fs::create_dir_all(&info)?;
    let file = info.join("exclude");
    let content = std::fs::read_to_string(&file).unwrap_or_default();
    if content.lines().any(|l| l.trim() == pattern) {
        return Ok(());
    }
    let sep = if content.is_empty() || content.ends_with('\n') { "" } else { "\n" };
    std::fs::write(&file, format!("{}{}{}\n", content, sep, pattern))?;
    Ok(())
}
