//! End-to-end: item commands on a plain store.

mod common;

use common::*;

/// ids in the order a command lists them, via --json
fn json_ids(env: &Env, dir: &std::path::Path, args: &[&str]) -> Vec<String> {
    let out = env.cake_ok(dir, &[args, &["--json"]].concat());
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    v["items"].as_array().unwrap().iter().map(|i| i["id"].as_str().unwrap().to_string()).collect()
}

/// the json a command prints
fn json_value(env: &Env, dir: &std::path::Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&env.cake_ok(dir, &[args, &["--json"]].concat())).unwrap()
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
        assert!(item.contains("status: task\n") && !item.contains("closed:"), "{}", item);
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
    assert!(read_item(&d, "a").contains("status: inbox\n"), "unchanged");
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
    assert!(a.contains("due: 2026-10-30\n") && a.contains("priority: 1\n") && a.contains("owner: "), "{}", a);
    assert!(a.ends_with("---\n# from stdin\n\nbody\n"), "{}", a);

    assert!(a.contains("status: task\n"), "{}", a);

    env.cake_ok(&d, &["add", "n", "-m", "# a note"]);
    assert!(read_item(&d, "n").contains("status: inbox\n"), "default is the inbox");

    env.cake_ok(&d, &["todo", "n"]);
    assert!(read_item(&d, "n").contains("status: task\n"), "sorted as a task");
    env.cake_ok(&d, &["note", "n"]);
    assert!(!read_item(&d, "n").contains("status:"), "a note has no status");
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

    assert!(d.join(".cake/items/a/c.md").is_file());
    env.cake_ok(&d, &["mv", "c", "b"]);
    assert!(read_item(&d, "c").contains("parent: \"[[b]]\"\n"));
    assert!(d.join(".cake/items/b/c.md").is_file());
    assert!(!d.join(".cake/items/a").exists());
    assert!(env.cake_err(&d, &["mv", "b", "c"]).contains("cycle"));
    env.cake_ok(&d, &["mv", "c", "-"]);
    assert!(!read_item(&d, "c").contains("parent:"));
    assert!(d.join(".cake/items/c.md").is_file());
}

#[test]
fn add_derives_a_slug_id_from_the_title() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "-m", "# Fix the merge driver for sync conflicts"]);
    env.cake_ok(&d, &["add", "-p", "fix", "-m", "# Fix the merge driver for sync conflicts"]);
    assert!(d.join(".cake/items/fix-merge-driver-sync.md").is_file());
    assert!(d.join(".cake/items/fix-merge-driver-sync/fix-merge-driver-sync-2.md").is_file());
    assert!(env.cake_err(&d, &["add", "a/b", "-m", "x"]).contains("not a valid id"));
}

#[test]
fn asks_block_work_until_answered_with_done_m() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "work", "--todo", "-m", "# Work"]);
    env.cake_ok(&d, &["add", "ask-db", "--todo", "-p", "work", "-t", "ask", "-m", "# Which db?\n\n- a) sqlite\n- b) files"]);
    env.cake_ok(&d, &["edit", "work", "--blocked-by", "ask-db"]);
    assert_eq!(json_ids(&env, &d, &["ready", "-t", "~ask"]), Vec::<String>::new());
    assert_eq!(json_ids(&env, &d, &["ready", "-t", "ask"]), vec!["ask-db"]);

    env.cake_ok(&d, &["done", "ask-db", "-m", "b, keep it plain"]);
    assert!(read_item(&d, "ask-db").ends_with("- b) files\n\nb, keep it plain\n"));
    assert_eq!(json_ids(&env, &d, &["ready", "-t", "~ask"]), vec!["work"]);
}

#[test]
fn ready_orders_by_score_deadline_before_prio() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "prio", "--todo", "--prio", "1", "-m", "# prio"]);
    env.cake_ok(&d, &["add", "late", "--todo", "--deadline", "2000-01-01", "-m", "# late"]);
    env.cake_ok(&d, &["add", "plain", "--todo", "-m", "# plain"]);
    env.cake_ok(&d, &["add", "key", "--todo", "-m", "# key"]);
    env.cake_ok(&d, &["add", "w1", "--todo", "--blocked-by", "key", "-m", "# w1"]);
    env.cake_ok(&d, &["add", "w2", "--todo", "--blocked-by", "w1", "-m", "# w2"]);
    env.cake_ok(&d, &["add", "w3", "--todo", "--blocked-by", "w1", "-m", "# w3"]);
    env.cake_ok(&d, &["add", "w4", "--todo", "--blocked-by", "w1", "-m", "# w4"]);
    assert_eq!(json_ids(&env, &d, &["ready"]), vec!["late", "key", "prio", "plain"]);
    assert!(read_item(&d, "late").contains("deadline: 2000-01-01\n"));
    env.cake_ok(&d, &["edit", "late", "--deadline", "-"]);
    assert!(!read_item(&d, "late").contains("deadline"));
    assert!(env.cake_err(&d, &["edit", "late", "--deadline", "soon"]).contains("deadline: 'soon' is no date"));
    env.cake_ok(&d, &["edit", "late", "--due", "+3d", "--deadline", "fri"]);
    let item = read_item(&d, "late");
    assert!(item.contains("due: 20") && item.contains("deadline: 20"), "stored as dates: {}", item);
}

#[test]
fn separate_store_is_found_from_its_dir_and_subtrees_move_between_stores() {
    let env = Env::new();
    let paper = env.path("paper");
    std::fs::create_dir(&paper).unwrap();
    let store = env.path("stores/paper");
    env.cake_ok(&paper, &["--store", store.to_str().unwrap(), "init"]);
    env.cake_ok(&paper.join(""), &["add", "outline", "-m", "# Outline"]);
    assert!(store.join("items/outline.md").is_file(), "found via the dirs of stores.json");

    let d = env.plain_store();
    env.cake_ok(&d, &["add", "idea", "-m", "# Idea"]);
    env.cake_ok(&d, &["add", "sub", "-p", "idea", "-m", "# Sub"]);
    let list = std::fs::read_to_string(env.path("config/cake/stores.json")).unwrap();
    assert!(list.contains("\"p\"") && list.contains("\"paper\""), "{}", list);

    env.cake_ok(&d, &["mv", "idea", "--to-store", "paper"]);
    assert!(store.join("items/idea/sub.md").is_file());
    assert!(!d.join(".cake/items/idea.md").exists() && !d.join(".cake/items/idea").exists());

    env.cake_ok(&paper, &["mv", "idea", "--to-store", "p"]);
    assert!(d.join(".cake/items/idea/sub.md").is_file());
    assert!(env.cake_err(&paper, &["mv", "outline", "--to-store", "nowhere"]).contains("not a store"));
    env.cake_ok(&d, &["add", "outline", "-m", "# Outline"]);
    assert!(env.cake_err(&paper, &["mv", "outline", "--to-store", "p"]).contains("already exists"));
}

#[test]
fn ascii_style_replaces_every_special_character() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "par", "--todo", "-m", "# parent"]);
    env.cake_ok(&d, &["add", "kid", "-p", "par", "-m", "# kid"]);
    env.cake_ok(&d, &["add", "end", "-p", "par", "--todo", "-m", "# end"]);
    env.cake_ok(&d, &["done", "end"]);
    let out = |style: &str| String::from_utf8(env.cake_env(&d, &["ls", "-A"], &[("CAKE_STYLE", style)]).stdout).unwrap();
    assert_eq!(out("ascii"), "o pa parent\n|- x en end\n\\- ? ki kid\n");
    assert_eq!(out("unicode"), "○ pa parent\n├─ ✓ en end\n└─ ◇ ki kid\n");
    let show = String::from_utf8(env.cake_env(&d, &["show", "par"], &[("CAKE_STYLE", "ascii")]).stdout).unwrap();
    assert!(show.is_ascii(), "{}", show);
}

#[test]
fn claude_hook_gives_tokens_to_the_item_whose_timer_ran() {
    let env = Env::new();
    let d = env.plain_store();
    let s = [("AI_AGENT", "claude-code"), ("CLAUDE_CODE_SESSION_ID", "s1")];
    env.cake_env(&d, &["add", "work", "--todo", "-m", "# Work"], &s);
    env.cake_env(&d, &["start", "work"], &s);
    let now = chrono::Utc::now();
    let line = |id: &str, t: chrono::DateTime<chrono::Utc>, model: &str| {
        format!(
            "{{\"type\":\"assistant\",\"timestamp\":\"{}\",\"message\":{{\"id\":\"{}\",\"model\":\"{}\",\"usage\":{{\"input_tokens\":10,\"output_tokens\":100,\"cache_read_input_tokens\":1000,\"cache_creation_input_tokens\":50}}}}}}\n",
            t.to_rfc3339(), id, model
        )
    };
    let transcript = env.path("t.jsonl");
    let old = now - chrono::Duration::hours(2); // before the timer: nobody's
    std::fs::write(&transcript, line("a", old, "m1") + &line("b", now, "m1")).unwrap();
    let hook = |cwd: &std::path::Path| {
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_cake"))
            .args(["usage", "claude-hook"])
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", env.path("config"))
            .env("XDG_DATA_HOME", env.path("data"))
            .env("XDG_STATE_HOME", env.path("state"))
            .env("AI_AGENT", "claude-code")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let input = format!("{{\"session_id\":\"s1\",\"transcript_path\":\"{}\"}}", transcript.display());
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), input.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    };
    hook(&d);
    assert!(!read_item(&d, "work").contains("usage:"), "running: still pending");

    std::thread::sleep(std::time::Duration::from_millis(1100));
    env.cake_env(&d, &["stop", "work"], &s);
    let item = read_item(&d, "work");
    assert!(item.contains("usage: {m1: {in: 10, out: 100, cache_read: 1000, cache_write: 50}}"), "{}", item);
    let show = env.cake_ok(&d, &["show", "work"]);
    assert!(show.contains("tokens 1.2k (m1)") && show.contains("Wh"), "{}", show);

    env.cake_ok(&d, &["usage", "add", "wo", "--model", "local", "--out", "7"]);
    assert!(read_item(&d, "work").contains("local: {in: 0, out: 7"));
}

#[test]
fn workflow_v2_inbox_task_question_review_and_parent() {
    let env = Env::new();
    let d = env.plain_store();
    let ai = [("AI_AGENT", "claude-code")];
    let ids = |args: &[&str]| json_ids(&env, &d, args);
    env.cake_ok(&d, &["add", "sync", "-m", "# Sync errors unclear"]);
    assert_eq!(ids(&["ls", "--status", "inbox"]), vec!["sync"]);
    env.cake_ok(&d, &["do", "sync"]);
    env.cake_ok(&d, &["edit", "sync", "--assignee", "ai"]);
    let item = read_item(&d, "sync");
    assert!(item.contains("status: task\n") && item.contains("assignee: ai\n") && item.contains("owner: "), "{}", item);

    // the agent splits: subtasks inherit the owner, a question blocks one
    env.cake_env(&d, &["add", "step", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# Name the step"], &ai);
    env.cake_env(&d, &["add", "hint", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# Hint per error"], &ai);
    env.cake_env(&d, &["add", "raw", "--task", "-p", "hint", "--assignee", "me", "-m", "# Raw message too?\n\n- a) yes\n- b) with -d"], &ai);
    env.cake_env(&d, &["edit", "hint", "--blocked-by", "raw"], &ai);
    assert!(read_item(&d, "step").contains("owner: "), "owner inherited");
    assert_eq!(ids(&["ready", "--assignee", "claude-code"]), vec!["step"], "parent waits, hint blocked");
    assert!(env.cake_err(&d, &["edit", "step", "--blocked-by", "sync"]).contains("deadlock") || env.cake_err(&d, &["edit", "step", "--blocked-by", "sync"]).contains("waits for it"));

    // step goes to review, its sibling continues once the question is answered
    env.cake_env(&d, &["review", "step", "-m", "named"], &ai);
    env.cake_ok(&d, &["done", "raw", "-m", "b) with -d"]);
    assert_eq!(ids(&["ready", "--assignee", "claude-code"]), vec!["hint"], "the answered question is no part of hint's work");
    env.cake_ok(&d, &["edit", "step", "--comment", "shorter please"]);
    let step = read_item(&d, "step");
    assert!(step.contains("status: task\n") && step.contains("comment (") && step.contains("): shorter please\n"), "{}", step);

    // all parts done: the parent goes to review by itself
    env.cake_ok(&d, &["done", "step", "hint"]);
    assert!(read_item(&d, "sync").contains("status: review\n"), "{}", read_item(&d, "sync"));
    env.cake_ok(&d, &["note", "sync"]);
    assert!(!read_item(&d, "sync").contains("status:"));
}

#[test]
fn agents_see_their_tasks_and_stop_when_reviews_are_full() {
    let env = Env::new();
    let d = env.plain_store();
    let ai = [("AI_AGENT", "claude-code")];
    let agent_ready = || {
        let out = env.cake_env(&d, &["ready", "--json"], &ai);
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        v["items"].as_array().unwrap().iter().map(|i| i["id"].as_str().unwrap().to_string()).collect::<Vec<_>>()
    };
    env.cake_ok(&d, &["add", "a", "--task", "--assignee", "ai", "-m", "# a"]);
    env.cake_ok(&d, &["add", "b", "--task", "--assignee", "claude-code", "-m", "# b"]);
    env.cake_ok(&d, &["add", "c", "--task", "--assignee", "me", "-m", "# c"]);
    env.cake_ok(&d, &["add", "r", "--task", "-m", "# r"]);
    env.cake_ok(&d, &["review", "r"]);
    assert_eq!(agent_ready(), vec!["a", "b"], "not the user's task");

    std::fs::write(d.join(".cake/config.hjson"), "{ max_review: 1 }").unwrap();
    assert!(agent_ready().is_empty());
    let out = env.cake_env(&d, &["start", "a", "--json"], &ai);
    assert!(String::from_utf8_lossy(&out.stdout).contains("waiting_for_review"));
    assert_eq!(json_ids(&env, &d, &["ready"]), vec!["a", "b", "c"], "the user still sees all ready work");

    std::fs::write(d.join(".cake/config.hjson"), "{ max_review: -1 }").unwrap();
    assert_eq!(agent_ready(), vec!["a", "b"]);
}

#[test]
fn migrate_moves_the_tag_workflow_and_is_idempotent() {
    let env = Env::new();
    let d = env.plain_store();
    let ai = [("AI_AGENT", "claude-code")];
    let items = d.join(".cake/items");
    std::fs::write(items.join("w.md"), "---\nid: w\nstatus: todo\nby: claude-code\n---\n# Work\n").unwrap();
    std::fs::create_dir_all(items.join("w")).unwrap();
    std::fs::write(items.join("w/q.md"), "---\nid: q\nstatus: todo\ntags: [ask]\nby: claude-code\n---\n# Which?\n").unwrap();
    std::fs::write(items.join("w/r.md"), "---\nid: r\nstatus: todo\ntags: [reply]\n---\n# shorter please\n").unwrap();
    std::fs::write(items.join("p.md"), "---\nid: p\nby: claude-code\n---\n# Plan\n\n- done when: x\n").unwrap();
    std::fs::write(items.join("idea.md"), "---\nid: idea\n---\n# raw idea\n").unwrap();
    let dry = env.cake_ok(&d, &["migrate"]);
    assert!(dry.contains("would change"), "{}", dry);
    assert!(std::fs::read_to_string(items.join("w.md")).unwrap().contains("status: todo"), "dry run writes nothing");
    env.cake_ok(&d, &["migrate", "--apply"]);
    let w = read_item(&d, "w");
    assert!(w.contains("status: task\n") && w.contains("assignee: claude-code\n") && w.contains("): shorter please"), "{}", w);
    let q = read_item(&d, "q");
    assert!(q.contains("assignee: ") && !q.contains("tags:"), "{}", q);
    assert!(!items.join("w/r.md").exists(), "comment item removed");
    assert!(read_item(&d, "p").contains("status: inbox\n"));
    assert!(read_item(&d, "idea").contains("status: inbox\n"));
    assert!(env.cake_env(&d, &["migrate"], &ai).status.success());
    let again = env.cake_ok(&d, &["migrate"]);
    assert!(again.contains("Nothing to migrate"), "{}", again);
}

#[test]
fn assignees_are_checked_against_the_store_config() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "t", "--task", "-m", "# t"]);
    env.cake_ok(&d, &["edit", "t", "--assignee", "anyone"]);
    std::fs::write(d.join(".cake/config.hjson"), "{ people: [\"bob\"], agents: [\"local\"] }").unwrap();
    assert!(env.cake_err(&d, &["edit", "t", "--assignee", "anyone"]).contains("unknown assignee 'anyone'"));
    env.cake_ok(&d, &["edit", "t", "--assignee", "local"]);
    env.cake_ok(&d, &["edit", "t", "--assignee", "-"]);
    assert!(!read_item(&d, "t").contains("assignee:"));
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
    assert_eq!(ids(&["ls", "--status", "inbox"]), vec!["bbb"]);
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

    env.cake_ok(&d, &["edit", "ccc", "--blocked-by", "aaa,bbb"]);
    assert!(read_item(&d, "ccc").contains("blocked_by: [\"[[aaa]]\", \"[[bbb]]\"]\n"));
    assert!(env.cake_err(&d, &["edit", "aaa", "--blocked-by", "ccc"]).contains("cycle"));

    assert_eq!(json_ids(&env, &d, &["ready"]), vec!["bbb", "aaa"], "prio 1 first, no blocked, no notes");

    env.cake_ok(&d, &["done", "aaa"]);
    env.cake_ok(&d, &["edit", "ccc", "--blocked-by", "~bbb"]);
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
    assert!(a.contains("status: task\ntags: [x]\n"), "{}", a);
    assert!(a.ends_with("---\n# new\n\nbody\n"), "{}", a);
    assert!(env.cake_err(&d, &["edit", "nope", "-m", "x"]).contains("not found"));
}

#[test]
fn edit_sets_and_clears_priority_and_due() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "a", "--todo", "-m", "# a"]);
    env.cake_ok(&d, &["edit", "a", "--prio", "1", "--due", "2026-10-30"]);
    let a = read_item(&d, "a");
    assert!(a.contains("due: 2026-10-30\n") && a.contains("priority: 1\n"), "{}", a);
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
            let t2 = s.spawn(|| env.cake(&a, &["edit", "a0", "--tag", &format!("t{}", i)]));
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
    run(&["edit", "x", "--tag", "a"], "s1");
    run(&["done", "x"], "s2");
    env.cake_ok(&d, &["edit", "x", "--tag", "b"]);
    assert!(read_item(&d, "x").contains("\nsessions: [s1, s2]\n"), "{}", read_item(&d, "x"));
}

#[test]
fn aliases_from_defaults_user_and_store_config() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "r", "--task", "-m", "# to review"]);
    env.cake_ok(&d, &["review", "r"]);
    env.cake_ok(&d, &["add", "i", "--note", "-m", "# idea"]);
    env.cake_ok(&d, &["add", "c", "-p", "i", "-m", "# child"]);

    let ids = |args: &[&str]| {
        let out = env.cake_ok(&d, args);
        ["r", "i", "c"]
            .into_iter()
            .filter(|id| out.lines().any(|l| l.split_whitespace().any(|w| w == *id)))
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&["check"]), vec!["r"]);
    assert_eq!(ids(&["inbox"]), vec!["c"], "i is a note");

    let user = env.path("config/cake/config.hjson");
    let cfg = std::fs::read_to_string(&user).unwrap().replace("aliases: {}", "aliases: {mine: \"ls -s idea\"}");
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
    run(&["add", "cd", "--todo", "-m", "# c"], &other);
    run(&["start", "ab"], &jul);
    run(&["start", "cd"], &other);
    run(&["add", "ef", "--todo", "-m", "# e"], &jul);
    run(&["add", "rv", "--task", "-m", "# r"], &other);
    run(&["review", "rv"], &other);
    assert_eq!(run(&["status"], &agent), "▶ ab a task with a rather lon… 0h00 · ▶ 1 others · ready 3 · review 1\n");

    let user = env.path("config/cake/config.hjson");
    let cfg = std::fs::read_to_string(&user).unwrap().replace(
        "status: [\"mine\", \"others\", \"ready\", \"today\", \"review\"]",
        "status: [\"ready\", \"review\"]",
    );
    std::fs::write(&user, cfg).unwrap();
    assert_eq!(run(&["status"], &jul), "ready 3 · review 1\n");
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
    env.cake_ok(&d, &["add", "b", "-p", "p", "--note", "-m", "# note"]);
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
    assert!(first.starts_with("*t* · task · #x · by ") && first.contains(" · prio 1"), "{}", first);
    assert!(out.ends_with("- bullet\n\n◇ n review note\n"), "{}", out);
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
    env.cake_ok(&d, &["edit", "km", "--blocked-by", "abx"]);
    assert!(read_item(&d, "abcdef").contains("status: done"));
    assert!(read_item(&d, "abxyzq").contains("parent: \"[[kmnpqr]]\""));
    assert!(read_item(&d, "kmnpqr").contains("blocked_by: [\"[[abxyzq]]\"]"));
    assert!(env.cake_ok(&d, &["show", "km"]).starts_with("*kmnpqr*"));

    let err = env.cake_err(&d, &["todo", "ab"]);
    assert!(err.contains("ab matches abcdef, abxyzq"), "{}", err);
    let out = env.cake(&d, &["todo", "ab", "--json"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("\"ambiguous_id\""));
}

#[test]
fn mine_selects_your_items_with_what_grew_under_them() {
    let env = Env::new();
    let d = env.plain_store();
    // the user is git user.name (or $USER); the agent only sets AI_AGENT
    let jul: [(&str, &str); 0] = [];
    let agent = [("AI_AGENT", "claude-code_x")];
    let run = |args: &[&str], e: &[(&str, &str)]| {
        let out = env.cake_env(&d, args, e);
        assert!(out.status.success(), "{}", stderr(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    run(&["add", "id", "-m", "# idea"], &jul);
    run(&["add", "pl", "-p", "id", "-m", "# plan"], &agent);
    run(&["add", "dc", "-t", "doc", "-m", "# doc"], &agent);
    // the agent runs cake for jul: --mine still means jul's items
    assert_eq!(run(&["ls", "--mine"], &agent), "◇ id idea\n└─ ◇ pl plan\n");
}

#[test]
fn rename_by_prefix_keeps_tree_and_blockers() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "qwxyzk", "--todo", "-m", "# goal"]);
    env.cake_ok(&d, &["add", "kid", "--todo", "-p", "qw", "-m", "# kid"]);
    env.cake_ok(&d, &["add", "other", "--todo", "--blocked-by", "qw", "-m", "# other, see [[qwxyzk]]"]);
    env.cake_ok(&d, &["rename", "qw", "agents"]);
    assert!(env.cake_err(&d, &["rename", "agents", "kid"]).contains("already exists"));
    assert_eq!(env.cake_ok(&d, &["ls"]), "○ ag goal\n└─ ○ ki kid\n○ ot other, see [[agents]]\n");
    assert!(read_item(&d, "other").contains("[[agents]]"));
}

#[test]
fn archive_moves_old_closed_subtrees_and_show_still_finds_them() {
    let env = Env::new();
    let d = env.plain_store();
    let items = d.join(".cake/items");
    env.cake_ok(&d, &["add", "cli", "--note", "-m", "# cli"]);
    env.cake_ok(&d, &["add", "old", "--todo", "-p", "cli", "-m", "# old work"]);
    env.cake_ok(&d, &["add", "sub", "--todo", "-p", "old", "-m", "# sub"]);
    env.cake_ok(&d, &["add", "doc", "--note", "-p", "old", "-m", "# doc"]);
    env.cake_ok(&d, &["add", "mixed", "--todo", "-p", "cli", "-m", "# mixed"]);
    env.cake_ok(&d, &["add", "kid", "--todo", "-p", "mixed", "-m", "# kid"]);
    env.cake_ok(&d, &["done", "sub", "old", "mixed"]);
    // closed just now: not older than 3 months
    assert_eq!(json_value(&env, &d, &["archive"])["archived"], serde_json::json!([]));
    // a closed parent with an open child stays; notes go along
    let dry = json_value(&env, &d, &["archive", "--older", "0d", "--dry-run"]);
    assert_eq!(dry["archived"], serde_json::json!(["doc", "old", "sub"]));
    assert!(items.join("cli/old.md").is_file());
    let out = json_value(&env, &d, &["archive", "--older", "0d"]);
    assert_eq!(out["archived"], serde_json::json!(["doc", "old", "sub"]));
    assert!(items.join("archive/cli/old.md").is_file());
    assert!(items.join("archive/cli/old/sub.md").is_file());
    assert!(!items.join("cli/old").exists());
    assert_eq!(env.cake_ok(&d, &["ls", "-A"]), "· cl cli\n└─ ✓ mi mixed\n   └─ ○ ki kid\n");
    assert!(env.cake_ok(&d, &["ls", "--archive"]).contains("old work"));
    assert!(env.cake_ok(&d, &["show", "ol"]).contains("old work"));
    assert!(env.cake_err(&d, &["add", "old", "-m", "# again"]).contains("archive"));
    assert!(env.cake_err(&d, &["add", "archive", "-m", "# reserved"]).contains("reserved"));
    // opened again: back among the open items
    env.cake_ok(&d, &["set", "task", "old"]);
    assert!(items.join("cli/old.md").is_file());
    assert!(env.cake_ok(&d, &["ls"]).contains("old work"));
}

#[test]
fn claude_hook_logs_session_time_split_by_the_running_timer() {
    let env = Env::new();
    let d = env.plain_store();
    let s = [("AI_AGENT", "claude-code"), ("CLAUDE_CODE_SESSION_ID", "s1")];
    env.cake_env(&d, &["add", "work", "--todo", "-m", "# Work"], &s);
    env.cake_env(&d, &["start", "work"], &s);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    let now = chrono::Utc::now();
    let line = |kind: &str, t: chrono::DateTime<chrono::Utc>| {
        format!("{{\"type\":\"{}\",\"timestamp\":\"{}\",\"message\":{{}}}}\n", kind, t.to_rfc3339())
    };
    let transcript = env.path("t.jsonl");
    // the user asked 10 minutes ago, the agent answers now
    std::fs::write(&transcript, line("user", now - chrono::Duration::minutes(10)) + &line("assistant", now)).unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_cake"))
        .args(["usage", "claude-hook"])
        .current_dir(&d)
        .env("XDG_CONFIG_HOME", env.path("config"))
        .env("XDG_DATA_HOME", env.path("data"))
        .env("XDG_STATE_HOME", env.path("state"))
        .env("AI_AGENT", "claude-code")
        .env("CAKE_AUTHOR", "jul")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!("{{\"session_id\":\"s1\",\"transcript_path\":\"{}\"}}", transcript.display());
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), input.as_bytes()).unwrap();
    assert!(child.wait().unwrap().success());
    let log = d.join(".cake/log");
    let files: Vec<_> = std::fs::read_dir(&log).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(files.len(), 1, "one file for this month and machine");
    let lines: Vec<serde_json::Value> =
        std::fs::read_to_string(&files[0]).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 2, "{:?}", lines);
    assert!(lines.iter().all(|l| l["who"] == "jul" && l["kind"] == "chat" && l["session"] == "s1"));
    assert!(lines[0].get("item").is_none(), "before the timer: the store only");
    assert_eq!(lines[1]["item"], "work");
}

#[test]
fn estimate_round_trips_and_clears() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "work", "--todo", "--est", "2h30m", "-m", "# Work"]);
    assert!(read_item(&d, "work").contains("estimate: 2h30m"));
    assert_eq!(json_value(&env, &d, &["show", "work"])["items"][0]["estimate"], 9000);
    assert!(env.cake_ok(&d, &["show", "work"]).contains("est 2h30m"));
    assert!(env.cake_err(&d, &["edit", "work", "--est", "soon"]).contains("estimate"));
    env.cake_ok(&d, &["edit", "work", "--est", "-"]);
    assert!(!read_item(&d, "work").contains("estimate"));
}

#[test]
fn stats_count_done_items_per_goal_and_period() {
    let env = Env::new();
    let d = env.plain_store();
    env.cake_ok(&d, &["add", "g", "--note", "-t", "goal", "-m", "# goal"]);
    env.cake_ok(&d, &["add", "t", "--todo", "-p", "g", "-m", "# task"]);
    env.cake_ok(&d, &["done", "t"]);
    let v = json_value(&env, &d, &["stats", "-p", "week"]);
    let s = &v["stores"][0]["stats"];
    assert_eq!(v["period"], "week");
    assert_eq!(s["done"], 1);
    assert_eq!(s["goals"][0]["id"], "g");
    assert!(env.cake_err(&d, &["stats", "-p", "decade"]).contains("no period"));
    assert!(env.cake_ok(&d, &["stats"]).contains("streak 1 d"));
}
