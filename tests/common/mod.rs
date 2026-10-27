//! Shared helpers: run the cake binary in temp git repos.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

pub struct Env {
    pub dir: TempDir,
}

impl Env {
    pub fn new() -> Self {
        let env = Env {
            dir: tempfile::tempdir().unwrap(),
        };
        let remote = env.path("remote.git");
        git(env.dir.path(), &["init", "-q", "--bare", "-b", "main", remote.to_str().unwrap()]);
        env
    }

    pub fn path(&self, p: &str) -> PathBuf {
        self.dir.path().join(p)
    }

    /// clones the remote into `name`, or creates the first repo with one code commit
    pub fn repo(&self, name: &str) -> PathBuf {
        let repo = self.path(name);
        let remote = self.path("remote.git");
        if git_ok(self.dir.path(), &["--git-dir", remote.to_str().unwrap(), "rev-parse", "-q", "--verify", "main"]) {
            git(self.dir.path(), &["clone", "-q", remote.to_str().unwrap(), repo.to_str().unwrap()]);
        } else {
            fs::create_dir(&repo).unwrap();
            git(&repo, &["init", "-q", "-b", "main"]);
            fs::write(repo.join("code.rs"), "fn main() {}\n").unwrap();
            git(&repo, &["add", "code.rs"]);
            git(&repo, &["commit", "-q", "-m", "code"]);
            git(&repo, &["remote", "add", "origin", remote.to_str().unwrap()]);
            git(&repo, &["push", "-q", "-u", "origin", "main"]);
        }
        repo
    }

    pub fn cake(&self, cwd: &Path, args: &[&str]) -> Output {
        self.cake_env(cwd, args, &[])
    }

    /// runs cake with extra env vars; agent and author vars of the caller are removed
    pub fn cake_env(&self, cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cake"))
            .args(args)
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", self.path("config"))
            .env("XDG_DATA_HOME", self.path("data"))
            .env_remove("CAKE_STORE")
            .env_remove("CAKE_AUTHOR")
            .env_remove("AI_AGENT")
            .env_remove("CAKE_SESSION")
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .envs(identity())
            .envs(envs.iter().copied())
            .output()
            .unwrap()
    }

    /// runs cake with `input` on stdin, expects success
    pub fn cake_stdin(&self, cwd: &Path, args: &[&str], input: &str) -> String {
        use std::io::Write;
        let mut child = Command::new(env!("CARGO_BIN_EXE_cake"))
            .args(args)
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", self.path("config"))
            .env("XDG_DATA_HOME", self.path("data"))
            .env_remove("CAKE_STORE")
            .env_remove("CAKE_AUTHOR")
            .env_remove("AI_AGENT")
            .env_remove("CAKE_SESSION")
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "cake {:?} failed: {}", args, stderr(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    pub fn cake_ok(&self, cwd: &Path, args: &[&str]) -> String {
        let out = self.cake(cwd, args);
        assert!(out.status.success(), "cake {:?} failed: {}", args, stderr(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    pub fn cake_err(&self, cwd: &Path, args: &[&str]) -> String {
        let out = self.cake(cwd, args);
        assert!(!out.status.success(), "cake {:?} should fail", args);
        stderr(&out)
    }
}

pub fn identity() -> [(&'static str, &'static str); 4] {
    [
        ("GIT_AUTHOR_NAME", "Cake Test"),
        ("GIT_AUTHOR_EMAIL", "cake-test@example.com"),
        ("GIT_COMMITTER_NAME", "Cake Test"),
        ("GIT_COMMITTER_EMAIL", "cake-test@example.com"),
    ]
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

pub fn git_ok(dir: &Path, args: &[&str]) -> bool {
    Command::new("git").args(args).current_dir(dir).envs(identity()).output().unwrap().status.success()
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(dir).envs(identity()).output().unwrap();
    assert!(out.status.success(), "git {:?} failed: {}", args, stderr(&out));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}


impl Env {
    /// plain store (no git) in `<tmp>/p`
    pub fn plain_store(&self) -> PathBuf {
        let dir = self.path("p");
        fs::create_dir(&dir).unwrap();
        self.cake_ok(&dir, &["init"]);
        dir
    }
}

pub fn read_item(dir: &Path, id: &str) -> String {
    fs::read_to_string(dir.join(".cake/items").join(format!("{}.md", id))).unwrap()
}
