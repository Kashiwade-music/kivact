//! Role: Thread enumeration and warning capture for isolated resource tests.

use std::sync::{Arc, Mutex, OnceLock};

pub(super) fn blocking_threads() -> Vec<String> {
    thread_names()
        .into_iter()
        .filter(|name| name.starts_with("kivact-blocking"))
        .collect()
}

#[cfg(target_os = "linux")]
fn thread_names() -> Vec<String> {
    std::fs::read_dir("/proc/self/task")
        .expect("process task list is readable")
        .filter_map(|entry| {
            let path = entry
                .expect("process task entry is readable")
                .path()
                .join("comm");
            match std::fs::read_to_string(path) {
                Ok(name) => Some(name.trim().to_owned()),
                // A worker may exit between directory enumeration and this read.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("process thread name is readable: {error}"),
            }
        })
        .collect()
}

#[cfg(windows)]
#[path = "windows.rs"]
mod windows;
#[cfg(windows)]
use windows::thread_names;

#[cfg(not(any(windows, target_os = "linux")))]
fn thread_names() -> Vec<String> {
    panic!("resource thread enumeration supports Windows and Linux");
}

pub(super) struct Warnings(pub(super) Arc<Mutex<Vec<String>>>);

#[derive(Default)]
struct Message {
    text: Option<String>,
    mapping: Option<String>,
    caller: Option<String>,
}
impl tracing::field::Visit for Message {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let slot = match field.name() {
            "message" => &mut self.text,
            "mapping" => &mut self.mapping,
            "caller" => &mut self.caller,
            _ => return,
        };
        *slot = Some(format!("{value:?}"));
    }
}
impl tracing::Subscriber for Warnings {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.target() == "kivact" && *metadata.level() == tracing::Level::WARN
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if self.enabled(event.metadata()) {
            let mut message = Message::default();
            event.record(&mut message);
            if let Some(mut text) = message.text {
                if let Some(mapping) = message.mapping {
                    text.push_str(&format!("; mapping={mapping}"));
                }
                if let Some(caller) = message.caller {
                    text.push_str(&format!("; caller={caller}"));
                }
                self.0
                    .lock()
                    .expect("test warning log is not poisoned")
                    .push(text);
            }
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

#[cfg(feature = "embed")]
pub(super) fn all_threads() -> Vec<String> {
    let mut names = thread_names();
    names.sort();
    names
}

#[cfg(feature = "embed")]
pub(super) fn owned_threads() -> Vec<String> {
    let mut names: Vec<_> = thread_names()
        .into_iter()
        .filter(|name| name.starts_with("arvane-") || name.starts_with("kivact-"))
        .collect();
    names.sort();
    names
}
#[cfg(all(feature = "embed", windows))]
pub(super) use windows::gui_resources;
#[cfg(all(feature = "embed", not(windows)))]
pub(super) fn gui_resources() -> (u32, u32) {
    (0, 0)
}

/// Capture warnings from the native UI thread as well as the test thread.
/// RESOURCE_TEST serializes readers; retain the history because module pinning
/// is deliberately a process-wide, once-only fact.
pub(super) fn warnings() -> Arc<Mutex<Vec<String>>> {
    static WARNINGS: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();
    WARNINGS
        .get_or_init(|| {
            let warnings = Arc::new(Mutex::new(Vec::new()));
            tracing::subscriber::set_global_default(Warnings(warnings.clone()))
                .expect("resource binary installs one warning subscriber");
            warnings
        })
        .clone()
}

#[cfg(feature = "embed")]
pub(super) fn retention_warnings(warnings: &[String]) -> usize {
    warnings
        .iter()
        .filter(|message| message.contains("module code kept mapped until process exit"))
        .count()
}
