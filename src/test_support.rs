//! Shared helpers for tests that need a real git repository and/or an
//! isolated config/data dir. Only compiled for `cfg(test)`.

use crate::config::Config;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// Isolates a test from the developer's real cwd and XDG config/data dirs,
/// restoring both on drop. `directories::ProjectDirs`/`BaseDirs` (used by
/// `util::config_path`/`default_save_file`) honor `XDG_CONFIG_HOME`/
/// `XDG_DATA_HOME` on Linux, so pointing those at a tempdir keeps tests from
/// touching the real `~/.config`/`~/.local/share`.
///
/// cwd and env vars are process-global, so any test using this must be
/// annotated `#[serial]` to avoid racing other sandboxed tests.
pub struct Sandbox {
    dir: TempDir,
    prev_cwd: PathBuf,
    prev_xdg_config: Option<String>,
    prev_xdg_data: Option<String>,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("create tempdir");
        let prev_cwd = env::current_dir().expect("read cwd");
        let prev_xdg_config = env::var("XDG_CONFIG_HOME").ok();
        let prev_xdg_data = env::var("XDG_DATA_HOME").ok();

        let config_dir = dir.path().join("config");
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();
        env::set_var("XDG_CONFIG_HOME", &config_dir);
        env::set_var("XDG_DATA_HOME", &data_dir);
        env::set_current_dir(dir.path()).expect("chdir into sandbox");

        Self {
            dir,
            prev_cwd,
            prev_xdg_config,
            prev_xdg_data,
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Initializes a git repo in the sandbox with one commit on a
    /// deterministically named `main` branch (independent of the host's
    /// `init.defaultBranch` config).
    pub fn init_git_repo(&self) {
        git("init", &["-q"]);
        git("symbolic-ref", &["HEAD", "refs/heads/main"]);
        git("config", &["user.email", "cake-test@example.com"]);
        git("config", &["user.name", "Cake Test"]);
        std::fs::write("README.md", "init\n").unwrap();
        git("add", &["README.md"]);
        git("commit", &["-q", "-m", "init"]);
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.prev_cwd);
        match &self.prev_xdg_config {
            Some(v) => env::set_var("XDG_CONFIG_HOME", v),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
        match &self.prev_xdg_data {
            Some(v) => env::set_var("XDG_DATA_HOME", v),
            None => env::remove_var("XDG_DATA_HOME"),
        }
    }
}

/// Creates a bare repo at `path` with its HEAD pointed at `refs/heads/main`,
/// so a later `git clone` checks out `main` regardless of the host's
/// `init.defaultBranch` config.
pub fn init_bare_remote(path: &Path) {
    git("init", &["-q", "--bare", path.to_str().unwrap()]);
    let status = Command::new("git")
        .arg("--git-dir")
        .arg(path)
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .status()
        .expect("spawn git");
    assert!(status.success());
}

/// Runs a git subcommand in the current directory, panicking with stderr on failure.
pub fn git(subcommand: &str, args: &[&str]) -> Output {
    let out = Command::new("git")
        .arg(subcommand)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {} {:?} failed: {}",
        subcommand,
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// Builds a default `Config` inside an already-active `Sandbox`.
pub fn default_config() -> Config {
    Config::new().expect("build default config")
}
