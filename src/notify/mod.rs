use notify_rust::Notification;
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn disable() {
    ENABLED.store(false, Ordering::Relaxed);
}
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

#[cfg(target_os = "linux")]
pub fn send(sum: &str, msg: &str) {
    if ENABLED.load(Ordering::Relaxed)
    {
        Notification::new()
            .summary(sum)
            .body(msg)
            .show()
            .map(|_x| 0)
            .unwrap_or(0);
    }
}
#[cfg(target_os = "macos")]
pub fn send(sum: &str, msg: &str) {
    if ENABLED.load(Ordering::Relaxed)
    {
        Notification::new()
            .summary(sum)
            .body(msg)
            .show()
            .map(|_| 0)
            .unwrap_or(0);
    }
}
#[cfg(other)]
pub fn send(sum: &str, msg: &str) {
    if ENABLED.load(Ordering::Relaxed)
    {
        println!("{}", sum);
        println!("{}", msg);
    }
}
