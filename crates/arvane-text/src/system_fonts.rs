//! Role: The process's system font scan, run once on its own worker thread.

use kivact_types::cell::SharedDeferred;
use kivact_types::process::DetachedThreads;
use parley::fontique::{Collection, CollectionOptions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// The system fonts, scanned once per process. The scan runs on the
/// worker `start` spawns; a caller that needs it before it ends waits for
/// the rest of it (`SharedDeferred` initializes once and makes later
/// callers wait); a caller that comes before any worker ran scans itself.
static SCAN: SharedDeferred<Collection> = SharedDeferred::new();

/// Whether the scan worker was started (once per process).
static STARTED: AtomicBool = AtomicBool::new(false);

/// Start the system font scan on its worker, once per process (PERF-10:
/// it overlaps window and GPU setup). The worker is detached: it is noted
/// in `DetachedThreads`, so an embedding host keeps the module loaded
/// while it may run (`arvane-native`'s module retention).
pub fn start_system_font_scan() {
    if STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("arvane-fonts".to_owned())
        .spawn(|| {
            SCAN.get_or_init(scan);
        });
    if spawned.is_ok() {
        DetachedThreads::note();
    }
}

/// A collection of the system fonts, waiting for the rest of the scan.
pub(crate) fn collection() -> Collection {
    let since = Instant::now();
    let collection = SCAN.get_or_init(scan).clone();
    tracing::debug!(
        target: "arvane_text::fonts",
        waited_us = since.elapsed().as_micros() as u64,
        "system fonts joined"
    );
    collection
}

fn scan() -> Collection {
    Collection::new(CollectionOptions {
        shared: false,
        system_fonts: true,
    })
}

// The scan crosses threads.
const _: fn() = || {
    fn shared<T: Send + Sync>() {}
    shared::<Collection>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_font_scan_worker_starts_once_and_is_noted_as_detached() {
        let threads: Vec<_> = (0..4)
            .map(|_| std::thread::spawn(start_system_font_scan))
            .collect();
        for thread in threads {
            thread.join().expect("start does not panic");
        }
        assert!(STARTED.load(Ordering::Acquire));
        assert!(DetachedThreads::any());
        let mut fonts = collection();
        assert!(
            fonts.family_names().next().is_some() || cfg!(not(any(windows, target_os = "linux")))
        );
    }
}
