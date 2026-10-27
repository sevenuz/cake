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
