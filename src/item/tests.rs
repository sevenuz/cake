use crate::item::*;

const FULL: &str = r#"---
id: k3f9a2
status: doing
tags: [cake, "two words"]
parent: "[[p0x7qe]]"
blocked_by: ["[[m2k8zz]]"]
related: ["[[a81hfq]]"]
created: 2026-10-22T09:12:00+02:00
closed: 2026-10-23T18:00:00+02:00
due: 2026-10-30
priority: 2
time:
  - 2026-10-22T09:15:00+02:00/2026-10-22T10:40:00+02:00
  - 2026-10-22T14:00:00+02:00/
custom: kept
other: [1, 2]
---
# File per item

body
"#;

#[test]
fn full_item_parses_and_round_trips_with_utc_offset_normalized() {
    let item = Item::from_markdown(FULL, "x", 0).unwrap();
    assert_eq!(item.id(), "k3f9a2");
    assert_eq!(item.parents(), ["p0x7qe".to_string()]);
    assert!(item.is_started());
    let again = Item::from_markdown(&item.to_markdown(), "x", 0).unwrap();
    assert_eq!(again.to_markdown(), item.to_markdown());
    assert!(
        item.to_markdown().contains("related:\n- '[[a81hfq]]'\ncustom: kept\nother:"),
        "unknown keys keep their order: {}",
        item.to_markdown()
    );
    assert!(item.to_markdown().ends_with("---\n# File per item\n\nbody\n"));
}

#[test]
fn missing_frontmatter_uses_defaults() {
    let item = Item::from_markdown("just text\r\n", "stem", 42).unwrap();
    assert_eq!(item.id(), "stem");
    assert_eq!(item.timestamp(), 42);
    assert_eq!(item.content(), "just text\n");
    assert!(!item.to_markdown().contains("status:"), "no status = note");
}

#[test]
fn malformed_known_field_is_an_error() {
    assert!(Item::from_markdown("---\nstatus: maybe\n---\n", "x", 0).is_err());
    assert!(Item::from_markdown("---\ntime: [nope]\n---\n", "x", 0).is_err());
}

#[test]
fn start_sets_doing_and_stop_closes_interval() {
    let mut item = Item::new("id3".to_string(), vec![], None, vec![]);
    item.start("me").unwrap();
    assert!(item.to_markdown().contains("status: doing\n"));
    item.stop("me").unwrap();
    assert!(item.is_stopped());
}

#[test]
fn print_shows_border_and_only_the_first_content_line() {
    let mut item = Item::new("abc".to_string(), vec![], None, vec![]);
    item.set_content("first line\nsecond line".to_string());
    let out = item.print(5, false);
    assert!(out.starts_with('|'));
    assert!(out.ends_with("first line"));
    assert!(!out.contains("second line"));
}

#[test]
fn print_uses_backslash_border_when_item_has_children() {
    let item = Item::new("abc".to_string(), vec![], None, vec![]);
    assert!(item.print(5, true).starts_with('\\'));
}

#[test]
fn done_stops_timer_and_sets_closed_reopen_clears_it() {
    let mut item = Item::new("a".into(), vec![], None, vec![]);
    item.start("me").unwrap();
    item.set_status(Status::Done).unwrap();
    assert!(item.is_stopped());
    assert!(item.to_markdown().contains("status: done\n"));
    assert!(item.to_markdown().contains("\nclosed: "));

    item.set_status(Status::Todo).unwrap();
    assert!(!item.to_markdown().contains("closed:"));
    assert_eq!(item.status(), Some(Status::Todo));
}

#[test]
fn note_from_old_kind_field_becomes_a_task_with_a_status() {
    let mut item = Item::from_markdown("---\nkind: note\n---\nx\n", "n", 0).unwrap();
    assert_eq!(item.status(), None);
    assert!(!item.to_markdown().contains("kind"), "old key dropped");
    item.set_status(Status::Todo).unwrap();
    assert!(item.to_markdown().contains("status: todo\n"));
}

#[test]
fn status_parse_rejects_unknown() {
    assert_eq!(Status::parse("dropped"), Ok(Status::Dropped));
    assert!(Status::parse("closed").is_err());
}

#[test]
fn add_fields_validate_and_serialize() {
    let mut item = Item::new("a".into(), vec![], None, vec![]);
    assert!(item.set_due(Some("tomorrow".into())).is_err());
    item.set_due(Some("2026-10-30".into())).unwrap();
    item.set_due(Some("2026-10-30T12:00:00+01:00".into())).unwrap();
    assert!(item.set_priority(Some(4)).is_err());
    item.set_priority(Some(1)).unwrap();
    item.set_blocked_by(vec!["b".into()]);
    let md = item.to_markdown();
    assert!(!md.contains("status:"), "{}", md);
    assert!(md.contains("blocked_by: [\"[[b]]\"]\n"), "{}", md);
    assert!(md.contains("due: \"2026-10-30T12:00:00+01:00\"\npriority: 1\n"), "{}", md);
}

#[test]
fn title_and_json_fields() {
    let mut item = Item::from_markdown(FULL, "x", 0).unwrap();
    assert_eq!(item.title(), "File per item");
    let json = item.to_json();
    assert_eq!(json["status"], "doing");
    assert_eq!(json["parent"], "p0x7qe");
    assert_eq!(json["running"], true);
    assert_eq!(json["time"][0]["end"], "2026-10-22T10:40:00+02:00");
    assert!(json["time"][1]["end"].is_null());
    assert_eq!(json["extra"]["custom"], "kept");
    assert!(json["tracked_seconds"].as_i64().unwrap() > 85 * 60);

    item.set_content("\nplain first line\nmore".into());
    assert_eq!(item.title(), "plain first line");
    let note = Item::new("n".into(), vec![], None, vec![]);
    assert!(note.to_json()["status"].is_null());
}

#[test]
fn timers_belong_to_their_actor() {
    let mut item = Item::new("a".into(), vec![], None, vec![]);
    item.start("jul").unwrap();
    item.start("claude-code").unwrap();
    assert!(item.start("jul").is_err(), "one timer per actor");
    item.stop("claude-code").unwrap();
    assert!(item.runs_for("jul") && !item.runs_for("claude-code"));
    assert!(item.stop("claude-code").is_err());
    let md = item.to_markdown();
    assert!(md.contains("/ jul\n"), "{}", md);
    assert!(md.contains(" claude-code\n"), "{}", md);
    let again = Item::from_markdown(&md, "a", 0).unwrap();
    assert_eq!(again.to_markdown(), md);
    assert_eq!(again.to_json()["time"][1]["by"], "claude-code");
}

#[test]
fn intervals_without_actor_and_odd_names_round_trip() {
    let old = "---\ntime:\n  - 2026-10-22T14:00:00+02:00/\n---\n";
    let mut item = Item::from_markdown(old, "a", 0).unwrap();
    assert!(item.runs_for("anyone"), "old open interval counts for everyone");
    item.stop("anyone").unwrap();
    item.start("Jul A. #1").unwrap();
    let md = item.to_markdown();
    assert!(md.contains("\"2026") && md.contains(" Jul A. #1\"\n"), "quoted: {}", md);
    assert!(Item::from_markdown(&md, "a", 0).unwrap().runs_for("Jul A. #1"));
}
