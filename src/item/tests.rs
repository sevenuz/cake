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
    assert!(item.to_markdown().contains("custom: kept\n"));
    assert!(item.to_markdown().ends_with("---\n# File per item\n\nbody\n"));
}

#[test]
fn missing_frontmatter_uses_defaults() {
    let item = Item::from_markdown("just text\r\n", "stem", 42).unwrap();
    assert_eq!(item.id(), "stem");
    assert_eq!(item.timestamp(), 42);
    assert_eq!(item.content(), "just text\n");
    assert!(item.to_markdown().contains("status: todo\n"));
}

#[test]
fn malformed_known_field_is_an_error() {
    assert!(Item::from_markdown("---\nstatus: maybe\n---\n", "x", 0).is_err());
    assert!(Item::from_markdown("---\ntime: [nope]\n---\n", "x", 0).is_err());
}

#[test]
fn start_sets_doing_and_stop_closes_interval() {
    let mut item = Item::new("id3".to_string(), vec![], None, vec![]);
    item.start().unwrap();
    assert!(item.to_markdown().contains("status: doing\n"));
    item.stop().unwrap();
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
