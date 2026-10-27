//! End-to-end: item commands on a plain store.

mod common;

use common::*;

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
fn stop_without_id_stops_the_running_item() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "-m", "# a"]);
    env.cake_ok(&d, &["add", "b", "-m", "# b"]);
    env.cake_ok(&d, &["start", "a"]);

    assert!(env.cake_ok(&d, &["stop"]).contains("1 stopped"));
    assert!(!read_item(&d, "a").contains("/\n"));
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

    let ids = |args: &[&str]| {
        let out = env.cake_ok(&d, args);
        ["aaa", "bbb", "ccc", "ddd"]
            .into_iter()
            .filter(|id| out.contains(id))
            .collect::<Vec<_>>()
    };
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

    let out = env.cake_ok(&d, &["ready"]);
    let (a, b) = (out.find("aaa").unwrap(), out.find("bbb").unwrap());
    assert!(b < a, "prio 1 first: {}", out);
    assert!(!out.contains("ccc"));
    assert!(!out.contains("nnn"), "notes are never ready");

    env.cake_ok(&d, &["done", "aaa"]);
    env.cake_ok(&d, &["unblock", "ccc", "--by", "bbb"]);
    assert!(env.cake_ok(&d, &["ready"]).contains("ccc"));
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
    let out = env.cake_env(&a, &["done", "mine"], &[("CLAUDECODE", "1")]);
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
