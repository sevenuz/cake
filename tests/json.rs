//! End-to-end: --json prints exactly one JSON document on stdout.

mod common;

use common::*;
use serde_json::Value;
use std::path::Path;

fn json(env: &Env, dir: &Path, args: &[&str]) -> (bool, Value) {
    let mut args = args.to_vec();
    args.push("--json");
    let out = env.cake(dir, &args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let value: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("cake {:?}: no single json document ({}): {}", args, e, stdout));
    assert_eq!(value["v"], 1, "{}", value);
    (out.status.success(), value)
}

fn ok(env: &Env, dir: &Path, args: &[&str]) -> Value {
    let (success, value) = json(env, dir, args);
    assert!(success, "cake {:?} failed: {}", args, value);
    value
}

fn err_code(env: &Env, dir: &Path, args: &[&str]) -> String {
    let (success, value) = json(env, dir, args);
    assert!(!success, "cake {:?} should fail: {}", args, value);
    value["error"]["code"].as_str().unwrap().to_string()
}

fn ids(value: &Value) -> Vec<&str> {
    value["items"].as_array().unwrap().iter().map(|i| i["id"].as_str().unwrap()).collect()
}

#[test]
fn item_commands_report_items_after_the_change() {
    let env = Env::new();
    let d = env.plain_store();

    let a = ok(&env, &d, &["add", "aaa", "--todo", "--prio", "1", "-m", "# Alpha\n\nbody"]);
    let item = &a["items"][0];
    assert_eq!(item["id"], "aaa");
    assert_eq!(item["title"], "Alpha");
    assert_eq!(item["status"], "todo");
    assert_eq!(item["priority"], 1);
    assert_eq!(item["ready"], true);
    assert!(item["path"].as_str().unwrap().ends_with(".cake/items/aaa.md"));

    ok(&env, &d, &["add", "bbb", "--todo", "--blocked-by", "aaa", "-p", "aaa", "-m", "# B"]);
    ok(&env, &d, &["add", "nnn", "-m", "# a note"]);

    let ls = ok(&env, &d, &["ls"]);
    let all = ls["items"].as_array().unwrap();
    assert_eq!(all.len(), 3);
    let aaa = all.iter().find(|i| i["id"] == "aaa").unwrap();
    assert_eq!(aaa["blocks"], serde_json::json!(["bbb"]));
    assert_eq!(aaa["children"], serde_json::json!(["bbb"]));
    assert!(all.iter().find(|i| i["id"] == "nnn").unwrap()["status"].is_null());

    assert_eq!(ids(&ok(&env, &d, &["ready"])), vec!["aaa"]);

    let started = ok(&env, &d, &["start", "aaa"]);
    assert_eq!(started["items"][0]["running"], true);
    let done = ok(&env, &d, &["done", "aaa"]);
    assert_eq!(done["items"][0]["status"], "done");
    assert!(done["items"][0]["closed"].is_string());
    assert_eq!(ids(&ok(&env, &d, &["ready"])), vec!["bbb"]);

    assert_eq!(ids(&ok(&env, &d, &["show", "bbb"])), vec!["bbb"]);
    assert_eq!(ok(&env, &d, &["rm", "nnn"])["removed"], serde_json::json!(["nnn"]));
}

#[test]
fn errors_have_stable_codes() {
    let env = Env::new();
    let d = env.plain_store();
    ok(&env, &d, &["add", "aaa", "-m", "# a"]);

    assert_eq!(err_code(&env, &d, &["done", "nope"]), "not_found");
    assert_eq!(err_code(&env, &d, &["show", "nope"]), "not_found");
    assert_eq!(err_code(&env, &d, &["add", "aaa", "-m", "x"]), "invalid_input");
    assert_eq!(err_code(&env, &d, &["add", "x"]), "invalid_input", "no editor");
    assert_eq!(err_code(&env, &d, &["edit", "aaa"]), "invalid_input", "no editor");
    assert_eq!(err_code(&env, &d, &["ls", "--status", "closed"]), "invalid_input");
    assert_eq!(err_code(&env, &d, &["rm"]), "invalid_input");
    assert_eq!(err_code(&env, &d, &["bogus"]), "invalid_input");
    assert_eq!(err_code(&env, &d, &["sync"]), "invalid_input", "plain store");
}

#[test]
fn sync_reports_and_git_errors_are_coded() {
    let env = Env::new();
    let a = env.repo("a");
    ok(&env, &a, &["init"]);
    ok(&env, &a, &["add", "one", "--todo", "-m", "# one"]);
    assert_eq!(err_code(&env, &a, &["sync"]), "not_published");
    assert_eq!(ok(&env, &a, &["sync", "--publish"])["published"], "origin");

    let b = env.repo("b");
    ok(&env, &b, &["init"]);
    ok(&env, &a, &["tag", "one", "from-a"]);
    assert_eq!(ok(&env, &a, &["sync"])["synced"], true);
    ok(&env, &b, &["tag", "one", "from-b"]);

    assert_eq!(err_code(&env, &b, &["sync"]), "conflict");
    assert_eq!(err_code(&env, &b, &["add", "x", "-m", "x"]), "conflict");
    assert_eq!(err_code(&env, &b, &["sync", "--continue"]), "conflict");
    assert_eq!(ok(&env, &b, &["sync", "--abort"])["aborted"], true);
    assert_eq!(err_code(&env, &b, &["sync", "--publish", "nowhere"]), "git");
}
