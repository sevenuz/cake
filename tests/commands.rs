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

    env.cake_ok(&d, &["reopen", "a,b"]);
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
    assert!(read_item(&d, "a").contains("status: todo"));
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
    env.cake_ok(&d, &["add", "c", "-m", "# c"]);

    env.cake_stdin(
        &d,
        &["add", "a", "--body-file", "-", "--due", "2026-10-30", "--prio", "1", "--blocked-by", "b", "--related", "c"],
        "# from stdin\n\nbody\n",
    );
    let a = read_item(&d, "a");
    assert!(a.contains("blocked_by: [\"[[b]]\"]\nrelated: [\"[[c]]\"]\n"), "{}", a);
    assert!(a.contains("due: 2026-10-30\npriority: 1\n"), "{}", a);
    assert!(a.ends_with("---\n# from stdin\n\nbody\n"), "{}", a);

    env.cake_ok(&d, &["add", "n", "--note", "-m", "# a note"]);
    let n = read_item(&d, "n");
    assert!(n.contains("kind: note\n") && !n.contains("status:"), "{}", n);
}

#[test]
fn add_rejects_bad_input_without_writing() {
    let env = Env::new();
    let d = env.plain_store();
    assert!(env.cake_err(&d, &["add", "x"]).contains("--body-file"));
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
