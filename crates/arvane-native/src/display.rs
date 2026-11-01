//! Role: Initial window placement and display frame pacing.
//!
//! Invariant: the period is read from the monitor's reported refresh rate
//! when the window is created and when it moves to another monitor; an
//! unknown or implausible report keeps the period the window had (60 Hz
//! before any report).
//!
//! The refresh rate is read again only when `Moved` or a scale change
//! finds the window on a monitor of another identity, before the
//! `EnvironmentChanged` that announces it (`runtime.rs`). Windows reports
//! whole hertz, rounded down: such a report stands for the common rate
//! (60, 120, 144 ..) within the hertz above it ([`period_ns`]). Below 20
//! Hz or above 1000 Hz a report is implausible (Windows reports 1 Hz for a
//! display's "hardware default" rate).
//!
//! Limits: a change of the same monitor's refresh rate is not noticed
//! (nothing reads it every frame).

#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;

mod placement;
mod refresh;

pub(crate) use placement::{center_window, initial_window_position};
pub(crate) use refresh::{period_ns, refresh_rate_millihertz};
