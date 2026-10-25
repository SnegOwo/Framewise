use std::sync::atomic::AtomicBool;

use crate::db::Db;
use crate::tracker::Tracker;

pub struct AppState {
    pub db: Db,
    pub tracker: Tracker,
    pub quitting: AtomicBool,
    pub mini_pinned: AtomicBool,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            tracker: Tracker::new(),
            quitting: AtomicBool::new(false),
            mini_pinned: AtomicBool::new(false),
        }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
