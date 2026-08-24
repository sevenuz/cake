use crate::item::*;

#[test]
fn test_from_string() {
    let serialized = r#"| id | easycase|
|---|---|
| timestamp | Tue Mar  7 13:53:04 2023 +0100|
| last modified | Tue Oct 17 17:13:12 2023 +0200|
| tags | done, nice|
| timetrack | Tue Mar  7 13:55:06 2023 +0100, Tue Mar  7 13:55:20 2023 +0100, Tue Mar  7 13:56:42 2023 +0100, Tue Mar  7 13:56:47 2023 +0100, Mon Oct 16 21:51:53 2023 +0200, Mon Oct 16 21:52:00 2023 +0200, Tue Oct 17 09:23:05 2023 +0200, Tue Oct 17 09:29:44 2023 +0200, Tue Oct 17 17:03:37 2023 +0200, Tue Oct 17 17:13:12 2023 +0200|
| parents | frech|
| children | a76, 2c5|

# EasyCase
morgen wird fleißig geeasycased von zu hause, das wird mega :)

## subheader 1
```
fn burger() {
//some stuff
}
```
### subheader 2
[link](https://hayrave.de)
"#;
    let itm = Item::from_str(serialized).unwrap();
    assert_eq!(itm.print_long(true), serialized);
}

/// Writes an item out and reparses it, asserting the two textual forms match.
/// A weaker check than field-by-field equality, but avoids requiring
/// `PartialEq`/`Debug` on `Item` just for tests.
fn assert_round_trips(item: &Item) {
    let serialized = item.print_long(true);
    let reparsed = Item::from_str(&serialized).expect("reparse");
    assert_eq!(reparsed.print_long(true), serialized);
}

#[test]
fn round_trip_with_all_optional_fields_empty() {
    let item = Item::new("id1".to_string(), vec![], vec![], vec![]);
    assert_round_trips(&item);
}

#[test]
fn round_trip_with_populated_relations_tags_and_content() {
    let mut item = Item::new(
        "id2".to_string(),
        vec!["child1".to_string(), "child2".to_string()],
        vec!["parent1".to_string()],
        vec!["tag1".to_string(), "tag2".to_string()],
    );
    item.set_content("multi\nline\ncontent".to_string());
    assert_round_trips(&item);
}

#[test]
fn round_trip_with_timetrack_entries() {
    let mut item = Item::new("id3".to_string(), vec![], vec![], vec![]);
    item.start().unwrap();
    item.stop().unwrap();
    assert_round_trips(&item);
}

#[test]
fn print_shows_border_and_only_the_first_content_line() {
    let mut item = Item::new("abc".to_string(), vec![], vec![], vec![]);
    item.set_content("first line\nsecond line".to_string());
    let out = item.print(5, false);
    assert!(out.starts_with('|'));
    assert!(out.ends_with("first line"));
    assert!(!out.contains("second line"));
}

#[test]
fn print_uses_backslash_border_when_item_has_children() {
    let item = Item::new("abc".to_string(), vec![], vec![], vec![]);
    let out = item.print(5, true);
    assert!(out.starts_with('\\'));
}
