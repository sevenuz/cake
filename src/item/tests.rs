use crate::item::*;

#[test]
fn test_str_to_vec() {
    assert_eq!(str_to_vec("1, 2, 3"), vec!["1", "2", "3"]);
}

#[test]
fn test_vec_to_str() {
    assert_eq!(vec_to_str(&vec!["1", "2", "3"]), "1, 2, 3");
    assert_eq!(vec_to_str(&vec![1, 2, 3]), "1, 2, 3");
}

#[test]
fn test_ft() {
    assert_eq!(ft(1678197184), "Tue Mar  7 14:53:04 2023 +0100");
}

#[test]
fn test_tf() {
    assert_eq!(tf("Tue Mar  7 14:53:04 2023 +0100").unwrap(), 1678197184);
}

#[test]
fn test_hm() {
    assert_eq!(hms(1000), "16m40s");
}

#[test]
fn test_from_string() {
    let serialized = r#"| id | easycase|
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
