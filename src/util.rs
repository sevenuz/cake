use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn timestamp() -> Duration {
    let t = SystemTime::now();
    t.duration_since(UNIX_EPOCH).unwrap()
}

pub fn split_comma(s: String) -> Vec<String> {
    // return empty vector if input is ""
    if s == "" {
        return vec![];
    }
    // TODO improvement!!!
    let ca: Vec<&str> = s.split(",").collect();
    let mut vec: Vec<String> = Vec::new();
    ca.into_iter().for_each(|ll| {
        vec.push(ll.to_string());
    });
    return vec;
}

pub fn remove_comma(s: String) -> String {
    s.replace(",", "")
}

