//! Short-lived helper process for watcher integration tests.
//! Usage: debrief-watcher-probe [seconds]

use std::thread;
use std::time::Duration;

fn main() {
    let secs: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    thread::sleep(Duration::from_secs(secs));
}
