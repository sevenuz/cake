//! End-to-end: item commands on a plain store.

mod common;

use common::*;

/// ids in the order a command lists them, via --json
fn json_ids(env: &Env, dir: &std::path::Path, args: &[&str]) -> Vec<String> {
    let out = env.cake_ok(dir, &[args, &["--json"]].concat());
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    v["items"].as_array().unwrap().iter().map(|i| i["id"].as_str().unwrap().to_string()).collect()
}

#[test]
fn done_drop_reopen_set_status_and_closed() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);
    env.cake_ok(&d, &["add", "b", "-m", "# b"]);

    env.cake_ok(&d, &["start", "a"]);
    env.cake_ok(&d, &["done", "a"]);
    let a = read_item(&d, "a");
    assert!(a.contains("status: done\n") && a.contains("closed: "), "{}", a);
    assert!(!a.contains("/\n"), "timer stopped: {}", a);

    env.cake_ok(&d, &["drop", "b"]);
    assert!(read_item(&d, "b").contains("status: dropped\n"));

    env.cake_ok(&d, &["reopen", "a,b"]); // alias of todo
    for id in ["a", "b"] {
        let item = read_item(&d, id);
        assert!(item.contains("status: todo\n") && !item.contains("closed:"), "{}", item);
    }
}

#[test]
fn stop_without_id_stops_only_your_running_items() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);
    env.cake_ok(&d, &["add", "b", "-m", "# b"]);
    env.cake_ok(&d, &["start", "a"]);
    let agent = [("AI_AGENT", "claude-code_x")];
    assert!(env.cake_env(&d, &["add", "c", "-m", "# c"], &agent).status.success());
    assert!(env.cake_env(&d, &["start", "c"], &agent).status.success());

    assert!(env.cake_ok(&d, &["stop"]).contains("1 stopped"));
    assert!(!read_item(&d, "a").contains("/\n"));
    assert!(read_item(&d, "c").contains("/\n"), "the agent's item keeps running");
    assert!(env.cake_env(&d, &["stop"], &agent).status.success());
    assert!(!read_item(&d, "c").contains("/\n"));
}

#[test]
fn changing_commands_refuse_empty_selector_and_unknown_ids() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);

    for cmd in ["rm", "done", "drop", "reopen", "start"] {
        assert!(env.cake_err(&d, &[cmd]).contains("No items selected"), "{}", cmd);
        assert!(env.cake_err(&d, &[cmd, "nope"]).contains("nope not found"), "{}", cmd);
    }
    assert!(!read_item(&d, "a").contains("status:"));
}

#[test]
fn edit_needs_a_known_id_and_a_terminal() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);
    assert!(env.cake_err(&d, &["edit", "nope"]).contains("nope not found"));
    assert!(env.cake_err(&d, &["edit", "a"]).contains("terminal"));
    assert!(env.cake_err(&d, &["add", "a", "-m", "# again"]).contains("cake edit"));
}

#[test]
fn add_with_all_fields_and_body_from_stdin() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "b", "-m", "# b"]);

    env.cake_stdin(
        &d,
        &["add", "a", "--todo", "-m", "-", "--due", "2026-10-30", "--prio", "1", "--blocked-by", "b"],
        "# from stdin\n\nbody\n",
    );
    let a = read_item(&d, "a");
    assert!(a.contains("blocked_by: [\"[[b]]\"]\n"), "{}", a);
    assert!(a.contains("due: 2026-10-30\npriority: 1\n"), "{}", a);
    assert!(a.ends_with("---\n# from stdin\n\nbody\n"), "{}", a);

    assert!(a.contains("status: todo\n"), "{}", a);

    env.cake_ok(&d, &["add", "n", "-m", "# a note"]);
    assert!(!read_item(&d, "n").contains("status:"), "default is a note");

    env.cake_ok(&d, &["todo", "n"]);
    assert!(read_item(&d, "n").contains("status: todo\n"), "note promoted");
}

#[test]
fn add_rejects_bad_input_without_writing() {
    let env = Env::new();
    let d = env.plain_store();
    assert!(env.cake_err(&d, &["add", "x"]).contains("-m TEXT"));
    assert!(env.cake_err(&d, &["add", "x", "-m", "x", "--prio", "5"]).contains("priority"));
    assert!(env.cake_err(&d, &["add", "x", "-m", "x", "--due", "soon"]).contains("due"));
    assert!(env.cake_err(&d, &["add", "x", "-m", "x", "--blocked-by", "nope"]).contains("nope not found"));
    assert!(!d.join(".cake/items/x.md").exists());
}

#[test]
fn mv_reparents_refuses_cycles_and_detaches() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);
    env.cake_ok(&d, &["add", "b", "-m", "# b"]);
    env.cake_ok(&d, &["add", "c", "-p", "a", "-m", "# c"]);

    env.cake_ok(&d, &["mv", "c", "b"]);
    assert!(read_item(&d, "c").contains("parent: \"[[b]]\"\n"));
    assert!(env.cake_err(&d, &["mv", "b", "c"]).contains("cycle"));
    env.cake_ok(&d, &["mv", "c", "-"]);
    assert!(!read_item(&d, "c").contains("parent:"));
}

#[test]
fn ls_filters_by_text_status_and_blockers() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "aaa", "--todo", "-m", "# Worktree sync"]);
    env.cake_ok(&d, &["add", "bbb", "-m", "# Design notes"]);
    env.cake_ok(&d, &["add", "ccc", "--todo", "--blocked-by", "aaa", "-m", "# TUI"]);
    env.cake_ok(&d, &["add", "ddd", "-m", "# old"]);
    env.cake_ok(&d, &["done", "ddd"]);

    let ids = |args: &[&str]| json_ids(&env, &d, args);
    assert_eq!(ids(&["ls", "-s", "WORKTREE"]), vec!["aaa"]);
    assert_eq!(ids(&["ls", "--status", "none"]), vec!["bbb"]);
    assert_eq!(ids(&["ls", "--status", "done"]), vec!["ddd"]);
    assert_eq!(ids(&["ls", "--ready"]), vec!["aaa"]);
    assert_eq!(ids(&["ls", "--blocked"]), vec!["ccc"]);
    assert!(env.cake_err(&d, &["ls", "--status", "closed"]).contains("unknown status"));
}

#[test]
fn block_unblock_and_ready() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "aaa", "--todo", "--prio", "2", "-m", "# a"]);
    env.cake_ok(&d, &["add", "bbb", "--todo", "--prio", "1", "-m", "# b"]);
    env.cake_ok(&d, &["add", "ccc", "--todo", "-m", "# c"]);
    env.cake_ok(&d, &["add", "nnn", "-m", "# a note"]);

    env.cake_ok(&d, &["block", "ccc", "--by", "aaa,bbb"]);
    assert!(read_item(&d, "ccc").contains("blocked_by: [\"[[aaa]]\", \"[[bbb]]\"]\n"));
    assert!(env.cake_err(&d, &["block", "aaa", "--by", "ccc"]).contains("cycle"));

    assert_eq!(json_ids(&env, &d, &["ready"]), vec!["bbb", "aaa"], "prio 1 first, no blocked, no notes");

    env.cake_ok(&d, &["done", "aaa"]);
    env.cake_ok(&d, &["unblock", "ccc", "--by", "bbb"]);
    assert!(json_ids(&env, &d, &["ready"]).contains(&"ccc".to_string()));
}

#[test]
fn show_takes_an_id_or_a_file() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "aaa", "-m", "# item title"]);
    std::fs::write(d.join("README.md"), "# readme title\n").unwrap();

    assert!(env.cake_ok(&d, &["show", "aaa"]).contains("item title"));
    assert!(env.cake_ok(&d, &["show", "README.md"]).contains("readme title"));
    assert!(env.cake_err(&d, &["show", "nope"]).contains("no item id or file"));
}

#[test]
fn help_exits_zero_and_usage_errors_fail() {
    let env = Env::new();
    let d = env.plain_store();
    assert!(env.cake_ok(&d, &["--help"]).contains("Usage: cake"));
    assert!(env.cake_ok(&d, &["help", "add"]).contains("--todo"));
    assert!(env.cake_err(&d, &["bogus"]).contains("unrecognized subcommand"));
}

#[test]
fn by_records_the_creator_and_agents_tag_commits() {
    let env = Env::new();
    let a = env.repo("a");
    env.cake_ok(&a, &["init"]);

    let out = env.cake_env(&a, &["add", "mine", "-m", "# mine"], &[("CAKE_AUTHOR", "jul")]);
    assert!(out.status.success());
    let out = env.cake_env(&a, &["add", "ai", "-m", "# ai"], &[("AI_AGENT", "claude-code_2-1-308_agent")]);
    assert!(out.status.success());
    let out = env.cake_env(&a, &["done", "mine"], &[("AI_AGENT", "claude-code_x")]);
    assert!(out.status.success());

    assert!(read_item(&a, "mine").contains("\nby: jul\n"));
    assert!(read_item(&a, "ai").contains("\nby: claude-code\n"));
    let log = git(&a.join(".cake"), &["log", "--format=%s", "-3"]);
    assert_eq!(log, "cake done mine [claude-code]\ncake add ai -m # ai [claude-code]\ncake add mine -m # mine");

    let out = env.cake_ok(&a, &["ls", "--by", "claude-code"]);
    assert!(out.contains("ai") && !out.contains("mine"), "{}", out);
    env.cake_ok(&a, &["add", "plain", "-m", "# x"]);
    assert!(read_item(&a, "plain").contains("\nby: "), "falls back to git user or $USER");
}

#[test]
fn edit_with_message_replaces_only_the_body() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "--todo", "-t", "x", "-m", "# old"]);
    env.cake_stdin(&d, &["edit", "a", "-m", "-"], "# new\n\nbody\n");
    let a = read_item(&d, "a");
    assert!(a.contains("status: todo\ntags: [x]\n"), "{}", a);
    assert!(a.ends_with("---\n# new\n\nbody\n"), "{}", a);
    assert!(env.cake_err(&d, &["edit", "nope", "-m", "x"]).contains("not found"));
}

#[test]
fn edit_sets_and_clears_priority_and_due() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "--todo", "-m", "# a"]);
    env.cake_ok(&d, &["edit", "a", "--prio", "1", "--due", "2026-10-30"]);
    assert!(read_item(&d, "a").contains("due: 2026-10-30\npriority: 1\n"));
    assert!(env.cake_err(&d, &["edit", "a", "--prio", "1", "--due", "soon"]).contains("due"));
    assert!(read_item(&d, "a").contains("priority: 1\n"), "nothing written on error");
    env.cake_ok(&d, &["edit", "a", "--prio", "0", "--due", "-"]);
    let a = read_item(&d, "a");
    assert!(!a.contains("priority:") && !a.contains("due:") && a.ends_with("# a"), "{}", a);
}

#[test]
fn parallel_commands_wait_for_the_lock_and_all_commit() {
    let env = Env::new();
    let a = env.repo("a");
    env.cake_ok(&a, &["init"]);
    for i in 0..20 {
        let (x, y) = (format!("a{}", i), format!("b{}", i));
        std::thread::scope(|s| {
            let t1 = s.spawn(|| env.cake(&a, &["add", &x, "--todo", "-m", "# x"]));
            let t2 = s.spawn(|| env.cake(&a, &["tag", "a0", &format!("t{}", i)]));
            let t3 = s.spawn(|| env.cake(&a, &["add", &y, "--todo", "-m", "# y"]));
            for out in [t1.join().unwrap(), t3.join().unwrap()] {
                assert!(out.status.success(), "{}", stderr(&out));
            }
            let tag = t2.join().unwrap();
            assert!(tag.status.success() || i == 0, "{}", stderr(&tag)); // a0 may not exist yet
        });
    }
    let store = a.join(".cake");
    assert_eq!(git(&store, &["status", "--porcelain"]), "", "everything committed");
    assert_eq!(std::fs::read_dir(store.join("items")).unwrap().count(), 40);
    let a0 = read_item(&a, "a0");
    assert!((1..20).all(|i| a0.contains(&format!("t{}", i))), "no lost tag: {}", a0);
}

#[test]
fn agent_sessions_are_recorded_on_changed_items() {
    let env = Env::new();
    let d = env.plain_store();
    let run = |args: &[&str], s: &str| {
        let out = env.cake_env(&d, args, &[("CAKE_SESSION", s)]);
        assert!(out.status.success(), "{}", stderr(&out));
    };
    run(&["add", "x", "--todo", "-m", "# x"], "s1");
    run(&["tag", "x", "a"], "s1");
    run(&["done", "x"], "s2");
    env.cake_ok(&d, &["tag", "x", "b"]);
    assert!(read_item(&d, "x").contains("\nsessions: [s1, s2]\n"), "{}", read_item(&d, "x"));
}

#[test]
fn aliases_from_defaults_user_and_store_config() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "r", "--todo", "-t", "review", "-m", "# to review"]);
    env.cake_ok(&d, &["add", "i", "-m", "# idea"]);
    env.cake_ok(&d, &["add", "c", "-p", "i", "-m", "# child"]);

    let ids = |args: &[&str]| {
        let out = env.cake_ok(&d, args);
        ["r", "i", "c"]
            .into_iter()
            .filter(|id| out.lines().any(|l| l.split_whitespace().any(|w| w == *id)))
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&["review"]), vec!["r"]);
    assert_eq!(ids(&["ideas"]), vec!["c"], "i has a child");

    let user = env.path("config/cake/config.hjson");
    let cfg = std::fs::read_to_string(&user).unwrap().replace("\"aliases\": {}", "\"aliases\": {\"mine\": \"ls -s idea\"}");
    std::fs::write(&user, cfg).unwrap();
    assert_eq!(ids(&["mine"]), vec!["i", "c"], "ls shows children of matches");
    std::fs::write(d.join(".cake/config.hjson"), "{ aliases: { mine: \"ls -s child\" } }").unwrap();
    assert_eq!(ids(&["mine"]), vec!["c"], "store config wins");

    assert!(env.cake_ok(&d, &["--help"]).contains("mine     ls -s child"));
    std::fs::write(d.join(".cake/config.hjson"), "{ aliases: { ls: \"ls -s child\" } }").unwrap();
    assert_eq!(ids(&["ls"]), vec!["c"], "built-in overridden");
}

#[test]
fn status_shows_your_timers_others_ready_and_configured_tags() {
    let env = Env::new();
    let d = env.plain_store();
    let jul = [("CAKE_AUTHOR", "jul")];
    // an agent runs cake for jul: the user stays jul
    let agent = [("CAKE_AUTHOR", "jul"), ("AI_AGENT", "claude-code_x")];
    let other = [("CAKE_AUTHOR", "bob")];
    let run = |args: &[&str], e: &[(&str, &str)]| {
        let out = env.cake_env(&d, args, e);
        assert!(out.status.success(), "{}", stderr(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    assert_eq!(run(&["status"], &jul), "", "nothing to show");
    run(&["add", "ab", "--todo", "-m", "# a task with a rather long title"], &jul);
    run(&["add", "cd", "--todo", "-t", "review", "-m", "# c"], &other);
    run(&["start", "ab"], &jul);
    run(&["start", "cd"], &other);
    run(&["add", "ef", "--todo", "-m", "# e"], &jul);
    assert_eq!(run(&["status"], &agent), "▶ ab a task with a rather lon… 0h00 · ▶ 1 others · ready 3\n");

    let user = env.path("config/cake/config.hjson");
    let cfg = std::fs::read_to_string(&user).unwrap();
    let cfg = cfg.replace("\"status\": [\n    \"mine\",", "\"status\": [\n    \"tag:review\",\n    \"mine\",");
    std::fs::write(&user, cfg).unwrap();
    assert!(run(&["status"], &jul).starts_with("review 1 · ▶ ab"), "{}", run(&["status"], &jul));
    let json: serde_json::Value = serde_json::from_str(&run(&["status", "--json"], &jul)).unwrap();
    assert_eq!((json["user"].as_str(), json["others"].as_u64(), json["ready"].as_u64()), (Some("jul"), Some(1), Some(3)));
    assert_eq!(json["tags"]["review"], 1);
}

#[test]
fn ls_is_a_tree_and_hides_closed_items() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "p", "--todo", "-t", "x", "-m", "# parent"]);
    env.cake_ok(&d, &["add", "a", "--todo", "-p", "p", "--prio", "1", "-m", "# first"]);
    env.cake_ok(&d, &["add", "b", "-p", "p", "-m", "# note"]);
    env.cake_ok(&d, &["add", "c", "--todo", "-p", "a", "-m", "# deep"]);
    env.cake_ok(&d, &["add", "z", "--todo", "-m", "# closed"]);
    env.cake_ok(&d, &["done", "z"]);

    assert_eq!(
        env.cake_ok(&d, &["ls"]),
        "○ p parent #x\n├─ ○ a first\n│  └─ ○ c deep\n└─ · b note\n"
    );
    assert!(env.cake_ok(&d, &["ls", "-A"]).contains("✓ z closed"));
    assert_eq!(env.cake_ok(&d, &["ls", "-t", "x"]).lines().count(), 4, "children of matches");
    assert_eq!(env.cake_ok(&d, &["ls", "a"]), "○ a first\n└─ ○ c deep\n");
}

#[test]
fn show_has_a_meta_line_and_lists_children() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "t", "--todo", "-t", "x", "--prio", "1", "-m", "# task\n\n- bullet"]);
    env.cake_ok(&d, &["add", "n", "-p", "t", "-m", "# review note"]);
    let out = env.cake_ok(&d, &["show", "t"]);
    let first = out.lines().next().unwrap();
    assert!(first.starts_with("*t* · todo · #x · by ") && first.ends_with(" · prio 1"), "{}", first);
    assert!(out.ends_with("- bullet\n\n· n review note\n"), "{}", out);
}

#[test]
fn piping_one_cake_command_into_another_does_not_deadlock() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "x", "-m", "# old"]);
    let bin = env!("CARGO_BIN_EXE_cake");
    for _ in 0..2 {
        let out = std::process::Command::new("timeout")
            .arg("5")
            .arg("sh")
            .arg("-c")
            .arg(format!("(sleep 0.2; {b} show x --json) | jq -r .items[0].body | sed s/old/new/ | {b} edit x -m -", b = bin))
            .current_dir(&d)
            .env("XDG_CONFIG_HOME", env.path("config"))
            .env("XDG_DATA_HOME", env.path("data"))
            .env_remove("AI_AGENT")
            .env_remove("CAKE_SESSION")
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    }
    assert!(read_item(&d, "x").contains("---\n# new"), "{}", read_item(&d, "x"));
}

#[test]
fn empty_content_is_refused() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "x", "-m", "# keep"]);
    assert!(env.cake_err(&d, &["edit", "x", "-m", "  "]).contains("Empty content"));
    assert!(env.cake_err(&d, &["add", "y", "-m", ""]).contains("Empty content"));
    assert!(read_item(&d, "x").ends_with("# keep"));
}

#[test]
fn id_prefixes_work_everywhere_and_ls_shows_short_ids() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "kmnpqr", "--todo", "-m", "# parent"]);
    env.cake_ok(&d, &["add", "abcdef", "--todo", "-p", "km", "-m", "# one"]);
    env.cake_ok(&d, &["add", "abxyzq", "--todo", "--blocked-by", "abc", "-m", "# two"]);
    assert_eq!(env.cake_ok(&d, &["ls"]), "○ abx two\n○ km parent\n└─ ○ abc one\n");

    env.cake_ok(&d, &["done", "abc"]);
    env.cake_ok(&d, &["mv", "abx", "km"]);
    env.cake_ok(&d, &["block", "km", "--by", "abx"]);
    assert!(read_item(&d, "abcdef").contains("status: done"));
    assert!(read_item(&d, "abxyzq").contains("parent: \"[[kmnpqr]]\""));
    assert!(read_item(&d, "kmnpqr").contains("blocked_by: [\"[[abxyzq]]\"]"));
    assert!(env.cake_ok(&d, &["show", "km"]).starts_with("*kmnpqr*"));

    let err = env.cake_err(&d, &["todo", "ab"]);
    assert!(err.contains("ab matches abcdef, abxyzq"), "{}", err);
    let out = env.cake(&d, &["todo", "ab", "--json"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("\"ambiguous_id\""));
}
