//! Role: Opt-in frame timing log for benchmarks (`KIVACT_FRAME_LOG=<path>`).
//!
//! Unset, nothing is opened or recorded. Set, the host truncates the file
//! and appends one line per presented frame, written at once (the reader
//! may be the app itself, while it runs):
//!
//! ```text
//! # <key>=<value> ...                                   (host notes)
//! F <end_unix_ns> <work_ns> <scene_ns> <raster_ns> <present_ns>
//! ```
//!
//! `end_unix_ns` is the wall clock (`SystemTime`) when the frame was handed
//! to the platform, so an app can match it with its own wall clock
//! readings. `work_ns` is the host's time on the UI thread for that frame
//! (the span is described at the host's call site), and the other three
//! are its parts.
//!
//! The notes: each renderer change as Arvane reports it when it happens
//! (`Shell::on_renderer_change`, written as Arvane words it; the first,
//! the renderer the window starts with, after `host=arvane`, as in
//! `host=arvane renderer=cpu gpu=starting`; later ones for example
//! `renderer=gpu adapter=… init_ms=… configure_ms=…`, or `renderer=cpu`
//! with the reason), `frame_period_ns=<ns>` at start and whenever the
//! refresh period changes, `accessibility=on` the first time assistive
//! technology reads the tree, and `memory_warning`. Benchmarks parse them.

use std::fs::File;
use std::io::Write as _;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Times of one presented frame.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FrameTimes {
    /// The whole frame on the UI thread.
    pub(crate) work: Duration,
    /// Building the scene (paint list) from the tree.
    pub(crate) scene: Duration,
    /// Rasterizing the scene.
    pub(crate) raster: Duration,
    /// Handing the pixels to the window.
    pub(crate) present: Duration,
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

    /// Record a frame that was just handed to the platform.
    pub(crate) fn frame(&mut self, times: FrameTimes) {
        if self.file.is_none() {
            return;
        }
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        self.write(&format!(
            "F {end} {} {} {} {}\n",
            times.work.as_nanos(),
            times.scene.as_nanos(),
            times.raster.as_nanos(),
            times.present.as_nanos(),
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
