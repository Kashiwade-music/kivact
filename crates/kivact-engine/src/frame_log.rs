//! Role: Opt-in frame timing log for benchmarks (`KIVACT_FRAME_LOG=<path>`).
//!
//! Unset, nothing is opened or recorded. Set, the host truncates the file
//! and appends one line per presented frame, written at once (the reader
//! may be the app itself, while it runs):
//!
//! ```text
//! # <key>=<value> ...                                   (host notes)
//! F <end_unix_ns> <work_ns> <scene_ns> <raster_ns> <present_ns>
//! S <chunks_rastered> <pixels_rastered> <surfaces_reused> <damage_pixels> <targets>
//! ```
//!
//! `end_unix_ns` is the wall clock (`SystemTime`) when the frame was handed
//! to the platform, so an app can match it with its own wall clock
//! readings. `work_ns` is the host's time on the UI thread for that frame
//! (the span is described at the host's call site), and the other three
//! are its parts. The `S` line after it counts the work of that frame
//! (`FrameStats`).
//!
//! The notes: each renderer change as Arvane reports it when it happens
//! (the `arvane::Observer` callback, written as Arvane words it; the first,
//! the renderer the window starts with, after `host=arvane`, as in
//! `host=arvane renderer=cpu gpu=starting`; later ones for example
//! `renderer=gpu adapter=… init_ms=… configure_ms=…`, or `renderer=cpu`
//! with the reason), `frame_period_ns=<ns>` at start and whenever the
//! refresh period changes, `accessibility=on` the first time assistive
//! technology reads the tree, and `memory_warning`. Benchmarks parse them.

use crate::contract::FrameRecord;
use std::cell::RefCell;
use std::fs::File;
use std::io::Write as _;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The observer of Arvane's renderer changes that notes each in `log`
/// (benchmarks parse the `renderer=` notes; the first is the renderer
/// the window starts with).
pub(crate) fn renderer_logger(
    log: Rc<RefCell<FrameLog>>,
) -> impl FnMut(kivact_types::WindowId, &arvane::RendererChange) + 'static {
    move |_window, change| {
        let mut log = log.borrow_mut();
        match change {
            arvane::RendererChange::Start { .. } => log.note(&format!("host=arvane {change}")),
            _ => log.note(&change.to_string()),
        }
    }
}

/// The frame log of one host window; disabled unless the variable is set.
#[derive(Debug, Default)]
pub(crate) struct FrameLog {
    file: Option<File>,
}

impl FrameLog {
    /// The environment variable naming the log file.
    pub(crate) const VAR: &'static str = "KIVACT_FRAME_LOG";

    /// A log writing to `$KIVACT_FRAME_LOG`, or a disabled one.
    pub(crate) fn from_env() -> Self {
        let file = std::env::var_os(Self::VAR)
            .filter(|path| !path.is_empty())
            .and_then(|path| File::create(path).ok());
        FrameLog { file }
    }

    #[cfg(feature = "testing")]
    /// A log that records nothing.
    pub(crate) fn disabled() -> Self {
        FrameLog { file: None }
    }

    /// Whether frames are recorded.
    pub(crate) fn enabled(&self) -> bool {
        self.file.is_some()
    }

    /// Record a note line (`# <text>`), e.g. the renderer in use.
    pub(crate) fn note(&mut self, text: &str) {
        self.write(&format!("# {text}\n"));
    }

    /// Record a frame that was just handed to the platform, with the work
    /// it did.
    pub(crate) fn frame(&mut self, record: &FrameRecord) {
        if self.file.is_none() {
            return;
        }
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        self.write(&format!(
            "F {end} {} {} {} {}\n",
            record.work_ns, record.scene_ns, record.raster_ns, record.present_ns,
        ));
        let stats = &record.stats;
        self.write(&format!(
            "S {} {} {} {} {}\n",
            stats.chunks_rastered,
            stats.pixels_rastered,
            stats.surfaces_reused,
            stats.damage_pixels,
            stats.targets,
        ));
    }

    fn write(&mut self, line: &str) {
        // One write per line; a failing file stops the log, never the host.
        if let Some(file) = &mut self.file
            && file.write_all(line.as_bytes()).is_err()
        {
            self.file = None;
        }
    }
}
