//! Output mode: human text or one JSON document on stdout (`--json`).

use std::sync::atomic::{AtomicBool, Ordering};

static JSON: AtomicBool = AtomicBool::new(false);

pub fn set_json(json: bool) {
    JSON.store(json, Ordering::Relaxed);
}

pub fn json() -> bool {
    JSON.load(Ordering::Relaxed)
}

/// println! that stays silent in `--json` mode
#[macro_export]
macro_rules! say {
    ($($arg:tt)*) => {
        if !$crate::out::json() {
            println!($($arg)*);
        }
    };
}
